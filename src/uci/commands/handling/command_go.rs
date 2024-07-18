use crate::game::Board;
use log::info;
use rand::seq::SliceRandom;

pub fn handle_go(board: &mut Board) -> Option<String> {
    let moves = board.get_all_possible_moves();
    let best_move = moves.choose(&mut rand::thread_rng());

    match best_move {
        Some(best_move) => {
            info!("Engine Calculation best move: {}", best_move);
            Some(format!("bestmove {}", best_move.as_source_dest()))
        }
        _ => {
            info!("No moves found for the current position");
            None
        }
    }
}
