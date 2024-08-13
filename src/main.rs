#![allow(dead_code)]

use chess_bot::{game::Board, init};

fn main() {
    init();

    let board = Board::default();
    println!("{}", board);
}
