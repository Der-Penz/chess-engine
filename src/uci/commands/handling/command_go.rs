use crate::game::{board::move_gen::MoveGeneration, Board};
use log::info;
use rand::seq::SliceRandom;

pub fn handle_go(board: &Board) -> Option<String> {
    let moves = MoveGeneration::generate_all_moves(board);
    let best_move = moves.choose(&mut rand::thread_rng());

    match best_move {
        Some(best_move) => {
            info!("Engine Calculation best move: {}", best_move);
            Some(format!("bestmove {}", best_move.as_uci_notation()))
        }
        _ => {
            info!("No moves found for the current position");
            None
        }
    }
}
