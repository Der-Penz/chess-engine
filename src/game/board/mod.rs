use thiserror::Error;

use crate::game::{MoveType, Square};

use super::{
    match_piece, moves::CastleType, CastleRights, Color, DetailedMove, Move, Piece, PieceVariation,
};

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

#[derive(Debug, Error)]
pub enum MoveError {
    #[error("The source square {0} is empty")]
    EmptySource(Square),
    #[error("Invalid move")]
    InvalidMove,
    #[error("You cannot capture a King")]
    KingCapture,
    #[error("You cannot move to the same square")]
    SameSquare,
    #[error("No moves available from the source square")]
    NoMovesAvailable,
    #[error("Invalid piece move: {0} can not move from {1} to {2}")]
    InvalidPieceMove(Piece, Square, Square),
    #[error("It is not {0:?}'s turn to move")]
    WrongColor(Color),
    #[error("{1} does not have the right to castle {0:?}")]
    MissingCastleRight(CastleType, Color),
}

#[derive(Clone)]
pub struct Board {
    black_boards: [u64; 7],
    white_boards: [u64; 7],
    color_to_move: Color,
    castle_rights: CastleRights,
    en_passant: Option<Square>, //notes the square behind the pawn that can be captured en passant.
    // a value over 63 means no en passant
    // e.g if pawn moves from F2 to F4, F3 is the en passant square
    ply_clock: u8, //number of half moves since the last capture or pawn advance
    move_number: usize,
}

impl std::default::Default for Board {
    fn default() -> Self {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .expect("Invalid base FEN String")
    }
}

impl Board {
    pub fn empty() -> Self {
        Board {
            white_boards: [0, 0, 0, 0, 0, 0, 0],
            black_boards: [0, 0, 0, 0, 0, 0, 0],
            color_to_move: Color::WHITE,
            castle_rights: CastleRights::default(),
            en_passant: None,
            ply_clock: 0,
            move_number: 1,
        }
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

    /// Will check if the move is any kind of special move
    /// it won't check if the move is valid only if it satisfies minimal requirements for a special move
    /// if required checks don't determine any special move, it will return MoveType::Normal
    fn get_move_type(
        &self,
        mov: &Move,
        source_piece: &Piece,
        dest_piece: &Option<Piece>,
    ) -> MoveType {
        if mov.is_null() || !mov.valid() {
            return MoveType::Normal;
        }

        if let Some(promotion_pice) = mov.promotion_piece() {
            return MoveType::Promotion(promotion_pice);
        }

        if source_piece.0 == PieceVariation::PAWN
            && self.en_passant.is_some_and(|sq| sq == mov.dest_sq())
            && ((source_piece.1 == Color::WHITE && mov.source_sq().rank() == 5 - 1)
                || (source_piece.1 == Color::BLACK && mov.source_sq().rank() == 4 - 1))
        {
            return MoveType::EnPassant;
        }

        if source_piece.0 == PieceVariation::KING {
            let castle_type =
                CastleType::satisfies_castle(&mov.source_sq(), &mov.dest_sq(), &source_piece.1);

            if let Some(castle_type) = castle_type {
                return MoveType::Castling(castle_type);
            }
        }

        MoveType::Normal
    }

    fn validate_move(
        &self,
        mov: &Move,
        source_piece: &Option<Piece>,
        dest_piece: &Option<Piece>,
        move_type: &MoveType,
    ) -> Result<(), MoveError> {
        if !mov.valid() {
            return Err(MoveError::InvalidMove);
        }

        if mov.is_null() {
            return Ok(());
        }

        if mov.source() == mov.dest() {
            return Err(MoveError::SameSquare);
        }

        let source_piece = source_piece
            .as_ref()
            .ok_or(MoveError::EmptySource(mov.source_sq()))?;

        if dest_piece.is_some_and(|p| p.0 == PieceVariation::KING) {
            return Err(MoveError::KingCapture);
        }

        if let MoveType::Castling(castle_type) = move_type {
            if !self.castle_rights.has(&source_piece.1, castle_type) {
                return Err(MoveError::MissingCastleRight(*castle_type, source_piece.1));
            }
        }

        //by comparing the move to the moves the engine itself generates, we can check if the move is valid and doesn't put the king in check
        //additional castling through check is already verified by the move generation
        let moves = self.get_pseudo_legal_moves(mov.source()).unwrap_or(vec![]);
        let moves = self.filter_legal_moves(moves);

        if moves.is_empty() {
            return Err(MoveError::NoMovesAvailable);
        }

        if !moves.contains(mov) {
            return Err(MoveError::InvalidPieceMove(
                *source_piece,
                mov.source_sq(),
                mov.dest_sq(),
            ));
        }

        Ok(())
    }

    /// plays a move on the board and returns a DetailedMove that can be used to undo the move
    /// if the move cannot be played, it will return a MoveError
    /// if full_validation is set to true, it will fully validate the move (king capture, pins, castling through check, invalid move for a piece, same color capture)
    /// if full_validation is set to false, it will only check basic requirements for a move (empty source, same square, invalid squares)
    /// if you now the move is from a trusted source, you can skip full_validation to improve performance
    pub fn play_move(
        &mut self,
        mov: &Move,
        full_validation: bool,
    ) -> Result<DetailedMove, MoveError> {
        if !mov.valid() {
            return Err(MoveError::InvalidMove);
        }
        if mov.is_null() {
            self.ply_clock = 0;
            if self.color_to_move == Color::BLACK {
                self.move_number += 1;
            }
            self.update_color_to_move();
            return Ok(DetailedMove::null());
        }

        let source_piece = self
            .get_field_piece(mov.source())
            .ok_or(MoveError::EmptySource(mov.source_sq()))?;
        let dest_piece = self.get_field_piece(mov.dest());

        if source_piece.1 != self.color_to_move {
            return Err(MoveError::WrongColor(source_piece.1));
        }
        let move_type = self.get_move_type(mov, &source_piece, &dest_piece);

        if full_validation {
            self.validate_move(mov, &source_piece.into(), &dest_piece, &move_type)?;
        }

        dest_piece.inspect(|p| {
            self.update_bb(p, mov.dest(), BitBoardOperation::RESET);
        });

        let castle_rights = self.castle_rights.clone();
        match move_type {
            MoveType::Normal => {
                self.update_bb(&source_piece, mov.source(), BitBoardOperation::RESET);
                self.update_bb(&source_piece, mov.dest(), BitBoardOperation::SET);

                if source_piece.0 == PieceVariation::KING {
                    self.castle_rights
                        .update(&source_piece.1, &CastleType::KingSide, false);
                    self.castle_rights
                        .update(&source_piece.1, &CastleType::QueenSide, false);
                }

                if source_piece.0 == PieceVariation::ROOK && castle_rights.has_any(&source_piece.1)
                {
                    match mov.source_sq() {
                        Square::A1 | Square::H1 => {
                            self.castle_rights.update(
                                &source_piece.1,
                                &CastleType::QueenSide,
                                false,
                            );
                        }
                        Square::A8 | Square::H8 => {
                            self.castle_rights.update(
                                &source_piece.1,
                                &CastleType::KingSide,
                                false,
                            );
                        }
                        _ => (),
                    }
                }
            }
            MoveType::Promotion(promotion_piece) => {
                self.update_bb(&source_piece, mov.source(), BitBoardOperation::RESET);
                self.update_bb(
                    &Piece::new(promotion_piece.into(), source_piece.1),
                    mov.dest(),
                    BitBoardOperation::SET,
                );
            }
            MoveType::EnPassant => {
                self.update_bb(&source_piece, mov.source(), BitBoardOperation::RESET);
                self.update_bb(&source_piece, mov.dest(), BitBoardOperation::SET);

                self.update_bb(
                    &Piece::new(PieceVariation::PAWN, source_piece.1.opposite()),
                    if source_piece.1 == Color::WHITE {
                        mov.dest() - 8
                    } else {
                        mov.dest() + 8
                    },
                    BitBoardOperation::RESET,
                );
            }
            MoveType::Castling(castle_type) => {
                self.update_bb(&source_piece, mov.source(), BitBoardOperation::RESET);
                self.update_bb(&source_piece, mov.dest(), BitBoardOperation::SET);

                self.update_bb(
                    &Piece::new(PieceVariation::ROOK, source_piece.1),
                    castle_type.get_rook_source(&source_piece.1).into(),
                    BitBoardOperation::RESET,
                );
                let rook_dest = castle_type.get_rook_dest(&source_piece.1).into();
                if let Some(remove) = self.get_field_piece(rook_dest) {
                    self.update_bb(&remove, rook_dest, BitBoardOperation::RESET);
                }

                self.update_bb(
                    &Piece::new(PieceVariation::ROOK, source_piece.1),
                    rook_dest,
                    BitBoardOperation::SET,
                );

                //reset castle rights
                self.castle_rights
                    .update(&source_piece.1, &CastleType::KingSide, false);
                self.castle_rights
                    .update(&source_piece.1, &CastleType::QueenSide, false);
            }
        }

        dest_piece.inspect(|p| {
            if p.0 == PieceVariation::ROOK {
                match mov.dest_sq() {
                    Square::A1 | Square::H1 => {
                        self.castle_rights
                            .update(&p.1, &CastleType::QueenSide, false);
                    }
                    Square::A8 | Square::H8 => {
                        self.castle_rights
                            .update(&p.1, &CastleType::KingSide, false);
                    }
                    _ => (),
                }
            }
        });

        let capture = dest_piece.map(|p| p.into());

        let ply_clock = self.ply_clock;
        if source_piece.0 == PieceVariation::PAWN {
            self.ply_clock = 0;

            //check if pawn moved 2 squares
            if ((mov.source() as i8) - (mov.dest() as i8)).abs() == 16 {
                self.en_passant = if source_piece.1 == Color::WHITE {
                    Some(Square::from(mov.dest() - 8))
                } else {
                    Some(Square::from(mov.dest() + 8))
                };
            } else {
                self.en_passant = None;
            }
        } else if capture.is_some() {
            self.ply_clock = 0;
        } else {
            self.ply_clock += 1;
        }

        if self.color_to_move == Color::BLACK {
            self.move_number += 1;
        }
        self.update_color_to_move();

        let check = self
            .in_check_color(self.color_to_move)
            .expect("Enemy Player should still have a king");

        Ok(DetailedMove::new(
            source_piece,
            mov.source(),
            mov.dest(),
            move_type,
            capture,
            check,
            ply_clock,
            castle_rights,
        ))
    }

    /// will undo the given move on the board.  
    /// Important: The move must be the last move played on the board or the board will be in an invalid state
    /// No validation is done to check if the move was last played or was valid
    /// program might panic with a invalid move
    pub fn undo_move(&mut self, last_move: &DetailedMove) {
        if last_move.color() == Color::BLACK {
            self.move_number -= 1;
        }
        self.en_passant = None;
        self.ply_clock = last_move.ply_clock();
        self.castle_rights = last_move.castle_rights();

        if last_move.is_null() {
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

                if last_move.piece().0 == PieceVariation::PAWN
                    && ((last_move.source() as i8) - (last_move.dest() as i8)).abs() == 16
                {
                    self.en_passant = if last_move.piece().1 == Color::WHITE {
                        Some(Square::from(last_move.dest() - 8))
                    } else {
                        Some(Square::from(last_move.dest() + 8))
                    };
                } else {
                    self.en_passant = None;
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

                self.en_passant = Some(Square::from(last_move.dest()));
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
                    (Color::WHITE, CastleType::KingSide) => (Square::H1, Square::F1),
                    (Color::WHITE, CastleType::QueenSide) => (Square::A1, Square::D1),
                    (Color::BLACK, CastleType::KingSide) => (Square::H8, Square::F8),
                    (Color::BLACK, CastleType::QueenSide) => (Square::A8, Square::D8),
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
        self.update_color_to_move();
    }
}
