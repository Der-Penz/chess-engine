pub mod bit_board;
pub mod board_error;
pub mod board_state;
pub mod display;
pub mod fen_utility;
pub mod move_gen;
pub mod zobrist;

use bit_board::BitBoard;
use board_error::{FENError, UndoMoveError};
use board_state::BoardState;
use fen_utility::FENUtility;
use log::error;
use move_gen::MoveGeneration;
use zobrist::ZOBRIST;

use super::{
    castle_rights::CastleType, color::Color, move_notation::Move, piece::Piece,
    piece_type::PieceType, square::Square,
};

#[derive(Clone)]
pub struct Board {
    bb_pieces: [[BitBoard; 6]; 2], //stores the bit boards for each piece type for each color
    bb_occupied: [BitBoard; 2],    //stores the occupied squares for each color
    bb_sliders: [BitBoard; 2],     //stores the occupied squares of sliding pieces for each color
    piece_square_table: [Option<PieceType>; 64], //quickly find the piece type on a square
    side_to_move: Color,
    ply_count: usize, //number of half moves played
    current_state: BoardState,
    previous_states: Vec<BoardState>, //stores the previous states of the board to be able to undo moves
    repetition_history: Vec<u64>, //stores the zobrist keys of the previous states to check for repetitions //TODO use a HashMap counter
}

impl std::default::Default for Board {
    fn default() -> Self {
        Board::from_fen(FENUtility::START_FEN).expect("Provided Fen::START_FEN is should be valid")
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", display::BoardDisplay::as_ascii(&self))
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", display::BoardDisplay::as_ascii(&self))?;
        writeln!(f, "Side to move: {}", self.side_to_move)?;
        writeln!(f, "Ply count: {}", self.ply_count)?;
        writeln!(f, "Current State: {:?}", self.current_state)?;
        writeln!(f, "Previous States count: {:?}", self.previous_states.len())?;
        writeln!(
            f,
            "Repetition History count: {:?}",
            self.repetition_history.len()
        )
    }
}

impl Board {
    fn empty() -> Self {
        Board {
            bb_pieces: [[BitBoard::default(); 6]; 2],
            bb_occupied: [BitBoard::default(); 2],
            bb_sliders: [BitBoard::default(); 2],
            piece_square_table: [None; 64],
            side_to_move: Color::White,
            ply_count: 0,
            current_state: BoardState::default(),
            previous_states: Vec::with_capacity(64),
            //allocate space for 64 half moves to avoid reallocation
            // (median game length is about 70 half moves  so it should only reallocate once if even required)
            repetition_history: Vec::with_capacity(32),
            //allocate space for 32 half moves to avoid reallocation (median game length is about 70 and irreversible moves reset the repetition history)
        }
    }

    pub fn cur_state(&self) -> &BoardState {
        &self.current_state
    }

    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    /// returns the color on the given square if occupied
    pub fn get_sq_color(&self, square: Square) -> Option<Color> {
        if self.bb_occupied[0].is_occupied(square) {
            Some(Color::White)
        } else if self.bb_occupied[1].is_occupied(square) {
            Some(Color::Black)
        } else {
            None
        }
    }

    /// returns the piece type on the given square if occupied
    pub fn get_sq_piece_variation(&self, square: Square) -> Option<PieceType> {
        self.piece_square_table[square]
    }

    /// returns the piece on the given square if occupied
    pub fn get_sq_piece(&self, square: Square) -> Option<Piece> {
        let color = self.get_sq_color(square);
        let piece_variation = self.get_sq_piece_variation(square);

        piece_variation
            .zip(color)
            .map(|(piece_variation, color)| Piece::new(piece_variation, color))
    }

    /// updates all state to reflect the given changes
    fn update_bb(&mut self, piece: Piece, square: Square, set: bool) {
        self.bb_pieces[piece.color()][piece.ptype()].update(square, set);
        self.bb_occupied[piece.color()].update(square, set);
        self.piece_square_table[square] = set.then_some(piece.ptype());
        if piece.ptype().is_sliding_piece() {
            self.bb_sliders[piece.color()].update(square, set);
        }
    }

    pub fn get_bb_pieces(&self) -> [[BitBoard; 6]; 2] {
        self.bb_pieces
    }

    pub fn get_bb_occupied(&self, color: Color) -> BitBoard {
        self.bb_occupied[color]
    }

    pub fn get_bb_for(&self, piece: Piece) -> BitBoard {
        self.bb_pieces[piece.color()][piece.ptype()]
    }

    pub fn get_bb_all_occupied(&self) -> BitBoard {
        self.bb_occupied[0] | self.bb_occupied[1]
    }

    pub fn get_king_pos(&self, color: Color) -> Square {
        self.bb_pieces[color][PieceType::King]
            .get_occupied()
            .next()
            .expect(&format!("{} King is missing", color))
    }

    pub fn get_piece_positions(&self, piece: Piece) -> impl Iterator<Item = Square> {
        self.bb_pieces[piece.color()][piece.ptype()].get_occupied()
    }

    pub fn get_bb_rook_slider(&self, color: Color) -> BitBoard {
        self.bb_pieces[color][PieceType::Rook] | self.bb_pieces[color][PieceType::Queen]
    }

    pub fn get_bb_bishop_slider(&self, color: Color) -> BitBoard {
        self.bb_pieces[color][PieceType::Bishop] | self.bb_pieces[color][PieceType::Queen]
    }

    ///Checks if the given square is attacked by the given color
    pub fn sq_attacked(&self, square: Square, color: Color) -> bool {
        let ally = self.get_bb_occupied(color.opposite());
        let enemy = self.get_bb_occupied(color);
        let bb = self.get_bb_pieces()[color];

        //check for attacks from non-sliding pieces
        let mut attacks = MoveGeneration::attacks_knight(square, *ally) & *bb[PieceType::Knight];
        attacks |= MoveGeneration::attacks_pawn(square, *enemy, *ally, color.opposite())
            & *bb[PieceType::Pawn];
        if attacks != 0 {
            return true;
        }

        //if no sliding pieces are available, we won't need to check for attacks
        if *self.bb_sliders[self.side_to_move.opposite()] != 0 {
            let attacks = MoveGeneration::attacks_bishop(square, *enemy, *ally)
                & (*bb[PieceType::Bishop] | *bb[PieceType::Queen]);
            if attacks != 0 {
                return true;
            }
            let attacks = MoveGeneration::attacks_rook(square, *enemy, *ally)
                & (*bb[PieceType::Rook] | *bb[PieceType::Queen]);
            if attacks != 0 {
                return true;
            }
        }

        false
    }

    pub fn in_check(&self) -> bool {
        self.sq_attacked(
            self.get_king_pos(self.side_to_move),
            self.side_to_move.opposite(),
        )
    }

    fn make_null_move(&mut self) {
        self.side_to_move = self.side_to_move.opposite();
        self.ply_count += 1;

        let mut new_zobrist_key = self.current_state.zobrist;

        new_zobrist_key ^= ZOBRIST.get_rn_side_to_move();
        new_zobrist_key ^= ZOBRIST.get_rn_en_passant(self.current_state.en_passant.as_ref());
        new_zobrist_key ^= ZOBRIST.get_rn_en_passant(None);

        let new_state = BoardState::new(
            new_zobrist_key,
            self.current_state.ply_clock + 1,
            None,
            self.current_state.castling_rights,
            None,
        );
        self.previous_states.push(self.current_state.clone());
        self.current_state = new_state;
    }

    fn undo_null_move(&mut self) -> Result<(), board_error::UndoMoveError> {
        self.current_state = self
            .previous_states
            .pop()
            .ok_or(UndoMoveError::NoMovesToUndo)?;
        self.side_to_move = self.side_to_move.opposite();
        self.ply_count -= 1;
        Ok(())
    }

    pub fn make_move(&mut self, mov: &Move, in_search: bool, validate: bool) -> Result<(), ()> {
        if mov.is_null() {
            self.make_null_move();
            return Ok(());
        }

        if validate {
            let moves = MoveGeneration::generate_legal_moves(self);
            if !moves.has(mov) {
                error!("Tried playing an invalid move");
                error!("Legal moves: {}", moves);
                error!("The illegal move: {}", mov);
                return Err(());
            }
        }

        let source = mov.source();
        let dest = mov.dest();
        let move_flag = mov.flag();
        let is_en_passant = move_flag.is_en_passant();
        let cur_state = self.current_state.clone();
        let mut new_zobrist = cur_state.zobrist;
        let mut new_castle_rights = cur_state.castling_rights.clone();

        let source_piece = self.get_sq_piece(source).unwrap();
        let move_color = source_piece.color();
        let dest_piece = if is_en_passant {
            Some(PieceType::Pawn.as_colored_piece(move_color.opposite()))
        } else {
            self.get_sq_piece(dest)
        };

        //handling captures
        if let Some(dest_piece) = dest_piece {
            if is_en_passant {
                let dest = dest - move_color.perspective() * 8;
                self.update_bb(dest_piece, dest, false);
            } else {
                self.update_bb(dest_piece, dest, false);
                new_zobrist ^= ZOBRIST.get_rn_piece(dest_piece.ptype(), dest);
            }
        }

        //handling promotions
        if move_flag.is_promotion() {
            self.update_bb(source_piece, source, false);
            new_zobrist ^= ZOBRIST.get_rn_piece(source_piece.ptype(), dest);

            let promoted_piece = move_flag
                .promotion_type()
                .expect("Move is flagged as promotion so it must have a promotion type")
                .as_colored_piece(move_color);
            self.update_bb(promoted_piece, dest, true);
            new_zobrist ^= ZOBRIST.get_rn_piece(promoted_piece.ptype(), dest);
        } else {
            //move the source piece to the destination (if promotion the source piece is already removed and promoted)
            self.update_bb(source_piece, source, false);
            new_zobrist ^= ZOBRIST.get_rn_piece(source_piece.ptype(), source);
            self.update_bb(source_piece, dest, true);
            new_zobrist ^= ZOBRIST.get_rn_piece(source_piece.ptype(), dest);
        }

        //handling king
        if source_piece.ptype() == PieceType::King {
            new_castle_rights.update(move_color, CastleType::KingSide, false);
            new_castle_rights.update(move_color, CastleType::QueenSide, false);

            if let Some(castle_type) = move_flag.castle_side() {
                let (rook_source, rook_dest) = castle_type.get_rook_positions(move_color);
                let rook = PieceType::Rook.as_colored_piece(move_color);
                self.update_bb(rook, rook_source, false);
                new_zobrist ^= ZOBRIST.get_rn_piece(rook.ptype(), rook_source);
                self.update_bb(rook, rook_dest, true);
                new_zobrist ^= ZOBRIST.get_rn_piece(rook.ptype(), rook_dest);
            }
        }

        //double pawn push
        let new_en_passant = if move_flag.is_double_pawn_push() {
            let en_passant_sq = dest - move_color.perspective() * 8;
            Some(en_passant_sq)
        } else {
            None
        };

        //update castle rights only if some were available
        if cur_state.castling_rights.as_u8() != 0 {
            //any piece move from or to a rooks starting position will remove the castle rights
            if source == Square::A1 || dest == Square::A1 {
                new_castle_rights.update(Color::White, CastleType::QueenSide, false);
            }
            if source == Square::H1 || dest == Square::H1 {
                new_castle_rights.update(Color::White, CastleType::KingSide, false);
            }
            if source == Square::A8 || dest == Square::A8 {
                new_castle_rights.update(Color::Black, CastleType::QueenSide, false);
            }
            if source == Square::H8 || dest == Square::H8 {
                new_castle_rights.update(Color::Black, CastleType::KingSide, false);
            }
        }

        //zobrist key update
        new_zobrist ^= ZOBRIST.get_rn_side_to_move();
        new_zobrist ^= ZOBRIST.get_rn_en_passant(cur_state.en_passant.as_ref());
        new_zobrist ^= ZOBRIST.get_rn_en_passant(new_en_passant.as_ref());
        if new_castle_rights != cur_state.castling_rights {
            new_zobrist ^= ZOBRIST.get_rn_castling(&self.current_state.castling_rights); // remove old castling rights state
            new_zobrist ^= ZOBRIST.get_rn_castling(&new_castle_rights); // add new castling rights state
        }

        let ply_clock = if source_piece.ptype() == PieceType::Pawn || dest_piece.is_some() {
            //TODO this will destroy move undoing for moves played on the board not in search
            if !in_search {
                self.repetition_history.clear();
            }
            0
        } else {
            cur_state.ply_clock + 1
        };
        self.current_state = BoardState::new(
            new_zobrist,
            ply_clock,
            new_en_passant,
            new_castle_rights,
            dest_piece.map(|p| p.ptype()),
        );
        if !in_search {
            self.repetition_history.push(cur_state.zobrist);
        }
        self.previous_states.push(cur_state);
        self.ply_count += 1;
        self.side_to_move = self.side_to_move.opposite();

        Ok(())
    }

    pub fn undo_move(
        &mut self,
        mov: &Move,
        in_search: bool,
    ) -> Result<(), board_error::UndoMoveError> {
        if mov.is_null() {
            return self.undo_null_move();
        }

        let captured_piece = self.current_state.captured_piece;
        self.current_state = self
            .previous_states
            .pop()
            .expect("No previous state to undo");
        self.ply_count -= 1;
        self.side_to_move = self.side_to_move.opposite();

        let source = mov.source();
        let dest = mov.dest();
        let move_flag = mov.flag();
        let color = self.side_to_move;

        let moved_piece = self
            .get_sq_piece(dest)
            .ok_or(UndoMoveError::InvalidLastMove)?;

        self.update_bb(moved_piece, dest, false);
        self.update_bb(moved_piece, source, true);

        //undo promotions ERROR TODO
        if let Some(promotion_type) = move_flag.promotion_type() {
            let promoted_piece = promotion_type.as_colored_piece(color);
            self.update_bb(promoted_piece, source, false);
            self.update_bb(PieceType::Pawn.as_colored_piece(color), source, true);
        }

        //undo captures and en passant
        if let Some(captured_piece) = captured_piece {
            let captured_piece = captured_piece.as_colored_piece(color.opposite());
            if move_flag.is_en_passant() {
                let captured_sq = dest - color.perspective() * 8;
                self.update_bb(captured_piece, captured_sq, true);
            } else {
                self.update_bb(captured_piece, dest, true);
            }
        }

        //undo castling
        if let Some(castle_type) = move_flag.castle_side() {
            let (rook_source, rook_dest) = castle_type.get_rook_positions(color);
            let rook = PieceType::Rook.as_colored_piece(color);
            self.update_bb(rook, rook_dest, false);
            self.update_bb(rook, rook_source, true);
        }

        if !in_search {
            self.repetition_history.pop();
        }

        Ok(())
    }

    /// Parses a FEN string and returns a Board
    pub fn from_fen(fen_string: &str) -> Result<Board, FENError> {
        fen_utility::FENUtility::from_fen(fen_string)
    }

    /// Returns a FEN string representation of the board.
    pub fn to_fen(&self) -> String {
        fen_utility::FENUtility::to_fen(self)
    }
}
