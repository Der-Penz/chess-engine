#![allow(dead_code)]

use chess_bot::{
    game::{
        board::{display::BoardDisplay, move_gen::MoveGeneration},
        Board,
    },
    init_logging,
};

fn main() {
    init_logging();

    let board =
        Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();

    println!("{}", board);

    let move_list = MoveGeneration::generate_legal_moves(&board);

    println!("{}", move_list);
    println!("{}", move_list.len());
    println!(
        "{}",
        BoardDisplay::as_board_with_attacks(&board, move_list.as_attack_bb())
    );
}
