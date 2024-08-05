use lazy_static::lazy_static;
use rand::{Rng, SeedableRng};

use crate::game::{
    castle_rights::CastleRights, color::Color, piece_type::PieceType, square::Square,
};

use super::Board;

lazy_static! {
    pub static ref ZOBRIST: Zobrist = Zobrist::new(0x8151894534);
}

pub struct Zobrist {
    pieces: [[u64; 64]; 12],
    castling: [u64; 16],
    en_passant: [u64; 9], // 8 files + 1 for no en passant
    side_to_move: u64,
}

impl Zobrist {
    pub fn new(seed: u64) -> Zobrist {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let mut pieces = [[0; 64]; 12];
        let mut castling = [0; 16];
        let mut en_passant = [0; 9];
        let side_to_move = rng.gen();

        for i in 0..12 {
            for j in 0..64 {
                pieces[i][j] = rng.gen();
            }
        }

        for i in 0..16 {
            castling[i] = rng.gen();
        }

        for i in 0..9 {
            en_passant[i] = rng.gen();
        }

        Zobrist {
            pieces,
            castling,
            en_passant,
            side_to_move,
        }
    }

    pub fn calculate_zobrist_key(&self, board: &Board) -> u64 {
        let mut zobrist_key = 0;
        PieceType::iter().for_each(|piece_type| {
            board.bb_pieces[Color::White][piece_type]
                .get_occupied()
                .for_each(|square| {
                    zobrist_key ^= self.pieces[piece_type][square];
                });
            board.bb_pieces[Color::Black][piece_type]
                .get_occupied()
                .for_each(|square| {
                    zobrist_key ^= self.pieces[piece_type as usize * 2][square];
                });
        });

        if board.side_to_move == Color::Black {
            zobrist_key ^= self.side_to_move;
        }

        zobrist_key ^= self.castling[board.current_state.castling_rights.as_u8() as usize];
        zobrist_key ^= self.get_rn_en_passant(board.current_state.en_passant.as_ref());

        zobrist_key
    }

    ///use for incremental update
    pub fn get_rn_side_to_move(&self) -> u64 {
        self.side_to_move
    }

    ///use for incremental update
    pub fn get_rn_piece(&self, piece_type: PieceType, square: Square) -> u64 {
        self.pieces[piece_type][square]
    }

    ///use for incremental update
    pub fn get_rn_castling(&self, castle_rights: &CastleRights) -> u64 {
        self.castling[castle_rights.as_u8() as usize]
    }

    ///use for incremental update
    pub fn get_rn_en_passant(&self, square: Option<&Square>) -> u64 {
        match square {
            Some(square) => self.en_passant[square.file() as usize],
            None => self.en_passant[8],
        }
    }
}
