use crate::game::Board;
use log::info;
use rand::seq::SliceRandom;

pub fn handle_go(board: &mut Board) -> Option<String> {
    info!("Handling Go Command");
    let moves = board.get_all_possible_moves();
    let best_move = moves.choose(&mut rand::thread_rng());
    let mut undo_stack = Vec::new();
    info!("Best Move: {:?}", best_move);
    match best_move {
        Some(m) => {
            let undo = board.play_move(m);
            undo_stack.push(undo);
            Some(format!("bestmove {}", m.as_source_dest()))
        }
        _ => None,
    }
}
