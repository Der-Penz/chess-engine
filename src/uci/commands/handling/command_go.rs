use crate::game::Board;
use log::info;
use rand::seq::SliceRandom;

pub fn handle_go(board: &mut Board) -> Option<String> {
    info!("Handling Go Command");
    let moves = board.get_all_possible_moves();
    let best_move = moves.choose(&mut rand::thread_rng());
    info!("Best Move: {:?}", best_move);
    match best_move {
        Some(m) => {
            board.move_piece(m.source(), m.dest());
            Some(format!("bestmove {}", m.as_source_dest()))
        }
        _ => None,
    }
}
