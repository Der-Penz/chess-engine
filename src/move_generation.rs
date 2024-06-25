use crate::board::{next_set_bit, Board, Color, PieceVariation};

pub fn generate_pseudo_legal_moves(game: &Board, color: Color) -> u64{

    let boards = game.get_boards(color);
    let mut moves : u64 = 0;

    for (piece, _i) in PieceVariation::iter(){
        if [PieceVariation::BISHOP, PieceVariation::QUEEN, PieceVariation::ROOK].contains(&piece){
            continue;
        }

        let mut board = boards[piece];
        while let Some(index) = next_set_bit(board){
            moves |= piece.attack_pattern(color)[index];
            board ^= 1 << index;
        }
    }


    return moves
}