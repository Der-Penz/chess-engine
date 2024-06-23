#[macro_use]
extern crate num_derive;
use board::Board;

mod board;
mod piece;
mod moves;
mod position;

fn main() {
    let mut board = Board::new();
    println!("{}", board);
    board.move_piece(8, 56);
    println!(" {}", board);
}
