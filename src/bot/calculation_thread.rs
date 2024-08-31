use std::sync::mpsc::{Receiver, Sender};

use crate::game::Move;

use super::{
    search::{AbortFlag, Search},
    ActionMessage, ReactionMessage,
};

pub fn thread_loop<S: Search + Send + 'static>(
    receiver: Receiver<ActionMessage>,
    sender: Sender<ReactionMessage>,
    flag: AbortFlag,
    mut search: S,
) {
    let tx = sender.clone();
    search.set_communication_channels(flag, tx);

    loop {
        let message = receiver.recv();

        match message {
            Ok(value) => match value {
                ActionMessage::Think(board, depth) => {
                    info!("Received search action");

                    let start_time = std::time::Instant::now();
                    let result = search.search(board, depth);
                    let elapsed = start_time.elapsed();

                    if let Some((best_move, eval)) = result {
                        info!("Best move: {:?} with eval: {}", best_move, eval);

                        sender.send(ReactionMessage::BestMove(best_move)).unwrap();
                    } else {
                        error!("No best move found, default to null move");
                        sender
                            .send(ReactionMessage::BestMove(Move::null()))
                            .unwrap();
                    }

                    info!("Search run {} seconds", elapsed.as_secs_f64());
                }
            },
            Err(_) => {
                info!("Disconnected from main thread");
                break;
            }
        };
    }
}
