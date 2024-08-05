use crate::game::{board::move_gen::MoveGeneration, Board};
use log::info;
use rand::Rng;

pub fn handle_go(board: &Board) -> Option<String> {
    let moves = MoveGeneration::generate_legal_moves(board);

    if moves.is_empty() {
        info!("No moves found for the current position");
        return None;
    }

    let idx = rand::thread_rng().gen_range(0..moves.len());
    let best_move = moves.get(idx).expect("Checked bounds must be valid");

    info!("Engine Calculation best move: {}", best_move);
    Some(format!("bestmove {}", best_move.as_uci_notation()))
}
