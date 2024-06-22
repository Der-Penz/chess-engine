#[macro_use]
extern crate num_derive;
use board::Board;

mod board;
mod piece;
mod moves;
mod position;

fn main() {
    let board = Board::new();
    println!("{}", board);
}
