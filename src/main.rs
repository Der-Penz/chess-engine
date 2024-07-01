use std::ops::Mul;

use game::{bit_manipulation::iter_set_bits, display_position, main_diagonal_to_1_rank, north, north_west, south, Board, A_FILE, MAIN_DIAGONAL};

#[macro_use]
extern crate num_derive;

mod game;
mod attack_pattern;

fn main() {
    let board= Board::base();
    println!("{}", board);
}
