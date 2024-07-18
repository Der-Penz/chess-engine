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

    pub fn move_piece(&mut self, source: u8, dest: u8) -> Option<DetailedMove> {
        let source_piece = self.get_field_piece(source)?;
        let dest_piece = self.get_field_piece(dest);

        let castle_move = self.is_castle_move(Square::from(source), Square::from(dest));
        if castle_move.is_some() {
            info!("Castle Move");
            todo!("handling castle move");
        } else {
            self.update_bb(&source_piece, source, BitBoardOperation::RESET);
            self.update_bb(&source_piece, dest, BitBoardOperation::SET);
        }

        if castle_move.is_some() {
            //TODO handle check check
            return Some(DetailedMove::new_castle(source_piece, source, dest, false));
        }

        let captured = match dest_piece {
            Some(dest_piece) => {
                self.update_bb(&dest_piece, dest, BitBoardOperation::RESET);
                Some(dest_piece.0)
            }
            None => None,
        };

        //TODO handle en passant promotion and check check
        Some(DetailedMove::new_normal(
            source_piece,
            source,
            dest,
            captured,
            false,
        ))
    }

    pub fn is_castle_move(&self, source: Square, dest: Square) -> Option<CastleType> {
        let castle_type = CastleType::matches_castle(source, dest);

        if castle_type.is_none() {
            return None;
        }

        let s_piece = self.get_field_piece(source.into())?;
        let d_piece = self.get_field_piece(dest.into())?;

        if s_piece.color_matches(d_piece)
            && s_piece.0 == PieceVariation::KING
            && d_piece.0 == PieceVariation::ROOK
        {
            Some(castle_type.unwrap())
        } else {
            None
        }
    }

    // pub fn play(&mut self, mov: &Move) -> Option<DetailedMove> {
    //     //TODO validate move
    //     let mov = self.move_piece(mov.source(), mov.dest());
    //     mov.as_ref().inspect(|mov| {
    //         self.update_color_to_move();
    //         match (CastleType::matches_king_side(mov.source_sq(), mov.dest_sq()), mov.color()) {
    //             (true, Color::WHITE) => {
    //                 self.white_castle.0 = false;
    //             }
    //             (true, Color::BLACK) => {
    //                 self.black_castle.0 = false;
    //             }
    //             _ => (),
    //         }
    //         match (CastleType::matches_queen_side(mov.source_sq(), mov.dest_sq()), mov.color()) {
    //             (true, Color::WHITE) => {
    //                 self.white_castle.1 = false;
    //             }
    //             (true, Color::BLACK) => {
    //                 self.black_castle.1 = false;
    //             }
    //             _ => (),
    //         };
    //     });
    //     mov
    // }

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
        todo!("Implement play_move")
    }

    pub fn undo_move(&mut self, last_move: &DetailedMove) {
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
