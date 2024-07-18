use log::info;

use crate::game::{MoveType, Square};

use super::{match_piece, moves::CastleType, Color, DetailedMove, Move, Piece, PieceVariation};

mod move_generation;
pub mod representation;

const PIECES_BOARD: usize = 6;

enum BitBoardOperation {
    SET,
    RESET,
}

impl BitBoardOperation {
    pub fn execute(&self, bb: &mut u64, square: u8) {
        match self {
            BitBoardOperation::SET => {
                *bb |= Square::to_board_bit(square);
            }
            BitBoardOperation::RESET => {
                *bb &= !Square::to_board_bit(square);
            }
        }
    }
}

#[derive(Clone)]
pub struct Board {
    black_boards: [u64; 7],
    white_boards: [u64; 7],
    color_to_move: Color,
    white_castle: (bool, bool), //(king side, queen side)
    black_castle: (bool, bool), // (king side, queen side)
    en_passant: Option<Square>, //notes the square behind the pawn that can be captured en passant.
    // a value over 63 means no en passant
    // e.g if pawn moves from F2 to F4, F3 is the en passant square
    half_move_clock: usize, //number of half moves since the last capture or pawn advance
    move_number: usize,
}

impl Board {
    pub fn empty() -> Self {
        Board {
            white_boards: [0, 0, 0, 0, 0, 0, 0],
            black_boards: [0, 0, 0, 0, 0, 0, 0],
            color_to_move: Color::WHITE,
            white_castle: (true, true),
            black_castle: (true, true),
            en_passant: None,
            half_move_clock: 0,
            move_number: 1,
        }
    }

    pub fn base() -> Self {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("Invalid base FEN String")
    }

    fn get_field_color(&self, square: u8) -> Option<Color> {
        if match_piece(square, self.black_boards[PIECES_BOARD]) {
            Some(Color::BLACK)
        } else if match_piece(square, self.white_boards[PIECES_BOARD]) {
            Some(Color::WHITE)
        } else {
            None
        }
    }

    fn get_field_piece_variation(&self, pos: u8) -> Option<PieceVariation> {
        for (piece, index) in PieceVariation::iter() {
            if match_piece(pos, self.black_boards[index] | self.white_boards[index]) {
                return Some(piece);
            }
        }
        None
    }

    pub fn get_field_piece(&self, square: u8) -> Option<Piece> {
        let color = self.get_field_color(square);
        let piece_variation = self.get_field_piece_variation(square);

        match color {
            Some(color) => Some(Piece(
                piece_variation.expect("Field must have a piece as it has a color"),
                color,
            )),
            None => None,
        }
    }

    fn update_bb(&mut self, piece: &Piece, square: u8, op: BitBoardOperation) {
        match piece {
            Piece {
                0: piece_variation,
                1: Color::WHITE,
            } => {
                op.execute(&mut self.white_boards[PIECES_BOARD], square);
                op.execute(&mut self.white_boards[*piece_variation], square);
            }
            Piece {
                0: piece_variation,
                1: Color::BLACK,
            } => {
                op.execute(&mut self.black_boards[PIECES_BOARD], square);
                op.execute(&mut self.black_boards[*piece_variation], square);
            }
        }
    }

    fn is_castle_move(
        source: Square,
        dest: Square,
        source_piece: &Piece,
        dest_piece: &Piece,
    ) -> Option<CastleType> {
        let castle_type = CastleType::satisfies_castle(&source, &dest, &source_piece.1);

        if castle_type.is_none() {
            return None;
        }

        if source_piece.color_matches(dest_piece)
            && source_piece.0 == PieceVariation::KING
            && dest_piece.0 == PieceVariation::ROOK
        {
            Some(castle_type.unwrap())
        } else {
            None
        }
    }

    fn is_promotion_move(dest: &Square, source_piece: &Piece) -> bool {
        source_piece.0 == PieceVariation::PAWN
            && ((source_piece.1 == Color::WHITE && dest.rank() == 8 - 1)
                || (source_piece.1 == Color::BLACK && dest.rank() == 1 - 1))
    }

    fn is_en_passant_move(
        source: &Square,
        dest: &Square,
        source_piece: &Piece,
        dest_piece: &Option<Piece>,
        en_passant: &Option<Square>,
    ) -> bool {
        en_passant.is_some_and(|sq| sq == *dest)
            && source_piece.0 == PieceVariation::PAWN
            && dest_piece.is_none()
            && (source.file() == 4 - 1 || source.file() == 5 - 1)
    }

    pub fn update_color_to_move(&mut self) -> Color {
        self.color_to_move = self.color_to_move.opposite();
        self.color_to_move
    }

    pub fn get_bbs_color(&self, color: &Color) -> [u64; 7] {
        match color {
            Color::WHITE => self.white_boards,
            Color::BLACK => self.black_boards,
        }
    }

    pub fn get_bb_color_occupied(&self, color: &Color) -> u64 {
        self.get_bbs_color(color)[6]
    }

    pub fn get_bb_for(&self, piece: &Piece) -> u64 {
        self.get_bbs_color(&piece.1)[piece.0]
    }

    pub fn get_bb_all_occupied(&self) -> u64 {
        self.white_boards[6] | self.black_boards[6]
    }

    pub fn play_move(&mut self, mov: &Move) -> Option<DetailedMove> {
        //TODO checking for same color capture
        if mov.is_null() {
            self.move_number += 1;
            self.update_color_to_move();
            return Some(DetailedMove::null());
        }

        if !mov.valid() {
            return None;
        }

        let source_piece = self.get_field_piece(mov.source())?;
        let dest_piece = self.get_field_piece(mov.dest());
        let mut move_type = MoveType::default();
        //move the piece, can be done no matter the move type
        self.update_bb(&source_piece, mov.source(), BitBoardOperation::RESET);
        self.update_bb(&source_piece, mov.dest(), BitBoardOperation::SET);

        if let Some(capture) = dest_piece.as_ref() {
            //castling
            if let Some(castle_type) =
                Board::is_castle_move(mov.source_sq(), mov.dest_sq(), &source_piece, capture)
            {
                self.update_bb(
                    &capture,
                    castle_type.get_rook_source(&capture.1).into(),
                    BitBoardOperation::RESET,
                );
                self.update_bb(
                    &capture,
                    castle_type.get_rook_dest(&capture.1).into(),
                    BitBoardOperation::SET,
                );

                move_type = MoveType::Castling(castle_type);
            } else {
                //normal capture
                self.update_bb(capture, mov.dest(), BitBoardOperation::RESET);
            }
        }

        if mov.promotion_piece().is_some()
            && Board::is_promotion_move(&mov.dest_sq(), &source_piece)
        {
            self.update_bb(&source_piece, mov.dest(), BitBoardOperation::RESET);
            self.update_bb(
                &Piece::new(mov.promotion_piece().unwrap().into(), source_piece.1),
                mov.dest(),
                BitBoardOperation::SET,
            );
            move_type = MoveType::Promotion(mov.promotion_piece().unwrap());
        }

        if Board::is_en_passant_move(
            &mov.source_sq(),
            &mov.dest_sq(),
            &source_piece,
            &dest_piece,
            &self.en_passant,
        ) {
            self.update_bb(
                &Piece::new(PieceVariation::PAWN, source_piece.1.opposite()),
                if source_piece.1 == Color::WHITE {
                    mov.dest() - 8
                } else {
                    mov.dest() + 8
                },
                BitBoardOperation::RESET,
            );
            move_type = MoveType::EnPassant;
        }

        self.update_color_to_move();

        let check = self
            .in_check_color(self.color_to_move)
            .expect("Enemy Player should still have a king");

        match move_type {
            MoveType::Normal => DetailedMove::new_normal(
                source_piece,
                mov.source(),
                mov.dest(),
                dest_piece.map(|p| p.into()),
                check,
            )
            .into(),
            MoveType::Promotion(p) => DetailedMove::new_promotion(
                source_piece,
                mov.source(),
                mov.dest(),
                p,
                dest_piece.map(|p| p.into()),
                check,
            )
            .into(),
            MoveType::EnPassant => {
                DetailedMove::new_en_passant(source_piece, mov.source(), mov.dest(), check).into()
            }

            MoveType::Castling(_) => {
                DetailedMove::new_castle(source_piece, mov.source(), mov.dest(), check).into()
            }
        }
    }

    pub fn undo_move(&mut self, last_move: &DetailedMove) {
        if last_move.is_null() {
            self.move_number -= 1;
            self.update_color_to_move();
            return;
        }

        match last_move.move_type() {
            MoveType::Normal => {
                //reset moved piece
                self.update_bb(
                    &last_move.piece(),
                    last_move.dest(),
                    BitBoardOperation::RESET,
                );
                self.update_bb(
                    &last_move.piece(),
                    last_move.source(),
                    BitBoardOperation::SET,
                );

                if let Some(capture) = last_move.capture() {
                    self.update_bb(
                        &Piece::new(capture, last_move.color().opposite()),
                        last_move.dest(),
                        BitBoardOperation::SET,
                    );
                }
            }
            MoveType::Promotion(promotion_piece) => {
                //reset moved piece
                self.update_bb(
                    &Piece::new(promotion_piece.into(), last_move.color()),
                    last_move.dest(),
                    BitBoardOperation::RESET,
                );
                self.update_bb(
                    &Piece::new(PieceVariation::PAWN, last_move.color()),
                    last_move.source(),
                    BitBoardOperation::SET,
                );

                if let Some(capture) = last_move.capture() {
                    self.update_bb(
                        &Piece::new(capture, last_move.color().opposite()),
                        last_move.dest(),
                        BitBoardOperation::SET,
                    );
                }
            }
            MoveType::EnPassant => {
                //reset moved piece
                self.update_bb(
                    &last_move.piece(),
                    last_move.dest(),
                    BitBoardOperation::RESET,
                );
                self.update_bb(
                    &last_move.piece(),
                    last_move.source(),
                    BitBoardOperation::SET,
                );

                let pawn_pos = match last_move.color() {
                    Color::WHITE => last_move.dest() - 8,
                    Color::BLACK => last_move.dest() + 8,
                };
                self.update_bb(
                    &Piece::new(PieceVariation::PAWN, last_move.color().opposite()),
                    pawn_pos,
                    BitBoardOperation::SET,
                );
            }
            MoveType::Castling(castle_type) => {
                self.update_bb(
                    &last_move.piece(),
                    last_move.dest(),
                    BitBoardOperation::RESET,
                );
                self.update_bb(
                    &last_move.piece(),
                    last_move.source(),
                    BitBoardOperation::SET,
                );

                let (rook_source, rook_dest) = match (last_move.color(), castle_type) {
                    (Color::WHITE, CastleType::KingSide) => {
                        self.white_castle.0 = true;
                        (Square::H1, Square::F1)
                    }
                    (Color::WHITE, CastleType::QueenSide) => {
                        self.white_castle.1 = true;
                        (Square::A1, Square::D1)
                    }
                    (Color::BLACK, CastleType::KingSide) => {
                        self.black_castle.0 = true;
                        (Square::H8, Square::F8)
                    }
                    (Color::BLACK, CastleType::QueenSide) => {
                        self.black_castle.1 = true;
                        (Square::A8, Square::D8)
                    }
                };

                self.update_bb(
                    &Piece::new(PieceVariation::ROOK, last_move.color()),
                    rook_dest.into(),
                    BitBoardOperation::RESET,
                );
                self.update_bb(
                    &Piece::new(PieceVariation::ROOK, last_move.color()),
                    rook_source.into(),
                    BitBoardOperation::SET,
                );
            }
        }

        match last_move.move_type() {
            MoveType::EnPassant => {
                self.en_passant = Some(Square::from(last_move.dest()));
            }
            _ => {
                self.en_passant = None;
            }
        }

        match last_move.capture().is_some() || last_move.piece_variation() == PieceVariation::PAWN {
            true => {
                todo!("reset counter to initial save it somewhere");
            }
            false => {
                self.half_move_clock = self.half_move_clock.checked_sub(1).unwrap_or(0);
            }
        }

        self.move_number -= 1;
        self.update_color_to_move();
    }
}
