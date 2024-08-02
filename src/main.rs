#![allow(dead_code)]

use chess_bot::{
    game::{self, Move},
    init_logging,
};
use log::error;

fn main() {
    init_logging();

    let mut game = game::Board::default();

    let mut moves = Vec::new();
    let played = "e2e4 e7e5 b1c3 g8f6 d2d4 e5d4 d1d4 b8c6 d4c4 c6e5 c4b5 d8e7 c1f4 c7c5 e1c1 a7a6 b5b6 c5c4 c1b1 f6g4 c3d5 e7h4 d5c7 e8d8 f4g3 h4f6 c7a8 f6b6 a8b6 c4c3 h2h3 g4f2 g3f2 c3b2 f2d4 f8d6 d4b2 d6c7 b6d5 h8e8 g1f3 f7f6 f1d3 d7d6 f3d4 b7b5 d4f5 g7g6 d5f6 e5d3 f6e8 d3b2 b1b2 d8e8 f5d6 c7d6 d1d6 e8e7 d6c6 e7d7 c6f6 d7e7 e4e5 c8b7 h1g1 a6a5 f6b6 b7e4 b6b5 a5a4 b5b4 e4c6 b4d4 h7h6 d4d6";
    let played = played.split_whitespace();
    let mut zobris = Vec::new();
    let mut zobris_undo = Vec::new();
    for play in played.into_iter() {
        let m = Move::from_uci_notation(play, &game).unwrap();
        println!("{m}");
        zobris.push(game.cur_state().zobrist);
        game.make_move(&m, false, true)
            .inspect_err(|_| error!("error making move: {}", m))
            .unwrap();
        moves.push(m);
    }

    zobris.push(game.cur_state().zobrist);
    println!("{}", game);

    while let Some(undo) = moves.pop() {
        zobris_undo.push(game.cur_state().zobrist);
        game.undo_move(&undo, false).unwrap();
    }
    zobris_undo.push(game.cur_state().zobrist);

    zobris_undo.reverse();
    assert_eq!(zobris, zobris_undo);
}
