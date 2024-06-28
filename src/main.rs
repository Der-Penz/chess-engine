use game::{Board};

#[macro_use]
extern crate num_derive;

mod game;
mod attack_pattern;

fn main() {
    let board= Board::base();
    println!("{}", board);
}
