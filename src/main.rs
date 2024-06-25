#[macro_use]
extern crate num_derive;
use board::Board;

mod board;
mod attack_pattern;

fn main() {
    // let mut board = Board::new();
    // board.move_piece(48, 24);
    // board.move_piece(57, 33);
    // println!("{}", board);
    // let moves = generate_pseudo_legal_moves(&board, Color::BLACK);
    // let moves = Board::only_pawns(moves, 0);
    // println!("{}", moves);
    // board.move_piece(8, 56);
    for (i,pos) in ATTACK_PATTERN_BISHOP.iter().enumerate(){
        println!("Board {i}");
        println!("{}", Board::only_pawns(*pos, to_board_bit(i as u8)));
        // println!("{}", Board::only_pawns(*pos, 0));
    }
}
