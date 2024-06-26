use game::{Board, Color};


#[macro_use]
extern crate num_derive;

mod game;
mod attack_pattern;

fn main() {
    let mut board = Board::new();
    println!("{}", board);
}
