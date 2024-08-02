use crate::game::{castle_rights::CastleRights, piece_type::PieceType, square::Square};

// BoardState is used to store the state metadata of the board to be able to undo moves
#[derive(Debug, Clone)]
pub struct BoardState {
    pub zobrist: u64,
    pub ply_clock: u8,
    pub en_passant: Option<Square>,
    pub castling_rights: CastleRights,
    pub captured_piece: Option<PieceType>,
}

impl std::default::Default for BoardState {
    fn default() -> Self {
        BoardState {
            zobrist: 0,
            ply_clock: 0,
            en_passant: None,
            castling_rights: CastleRights::default(),
            captured_piece: None,
        }
    }
}

impl BoardState {
    pub fn new(
        zobrist: u64,
        ply_clock: u8,
        en_passant: Option<Square>,
        castling_rights: CastleRights,
        captured_piece: Option<PieceType>,
    ) -> BoardState {
        BoardState {
            zobrist,
            ply_clock,
            en_passant,
            castling_rights,
            captured_piece,
        }
    }
}
