#![allow(dead_code)]

use chess_bot::{game::Board, init_logging};

fn main() {
    init_logging();

    let board = Board::default();

    println!("{}", board);
}
