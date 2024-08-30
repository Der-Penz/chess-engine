use std::sync::mpsc::{Receiver, Sender};

use super::{
    search::{AbortFlag, Search},
    ActionMessage, ReactionMessage,
};

pub fn thread_loop<S: Search + Send + 'static>(
    receiver: Receiver<ActionMessage>,
    mut sender: Sender<ReactionMessage>,
    flag: AbortFlag,
    mut search: S,
) {
    loop {
        let message = receiver.recv();

        match message {
            Ok(value) => match value {
                ActionMessage::Think(board, depth) => {
                    info!("Received search action");

                    let start_time = std::time::Instant::now();
                    let result = search.search(board, depth, &flag, &mut sender);
                    let elapsed = start_time.elapsed();

                    if let Some((best_move, eval)) = result {
                        info!("Best move: {:?} with eval: {}", best_move, eval);

                        sender.send(ReactionMessage::BestMove(best_move)).unwrap();
                    } else {
                        error!("No best move found");
                    }

                    info!(
                        "Search depth {depth} completed in {}s",
                        elapsed.as_secs_f64()
                    );
                }
            },
            Err(_) => {
                info!("Disconnected from main thread");
                break;
            }
        };
    }
}
