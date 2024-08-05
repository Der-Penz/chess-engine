#![allow(dead_code)]
#![allow(unused)]

use chess_bot::{
    game::{
        self,
        board::{
            bit_board::BitBoard,
            display::BoardDisplay,
            move_gen::{attack_pattern::direction_mask::DIRECTION_MASKS, MoveGeneration},
        },
        Board, Move,
    },
    init_logging,
};
use log::error;
use rand::Rng;

fn main() {
    init_logging();

    let mut game = game::Board::default();

    println!("{}", game);
}
