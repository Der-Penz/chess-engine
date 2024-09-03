use std::sync::mpsc::{Receiver, Sender};

use super::{
    search::{
        searcher::Searcher,
        transposition_table::{ReplacementStrategy, TranspositionTable},
        AbortFlag,
    },
    ActionMessage, ReactionMessage,
};

pub fn thread_loop(
    receiver: Receiver<ActionMessage>,
    sender: Sender<ReactionMessage>,
    flag: AbortFlag,
) {
    let tx = sender.clone();
    let tt = TranspositionTable::new(1024_f64, ReplacementStrategy::DepthPriority);
    let mut searcher = Searcher::new(tt, tx, flag);

    loop {
        match receiver.recv() {
            Ok(value) => match value {
                ActionMessage::Think(board, depth) => {
                    let start_time = std::time::Instant::now();
                    searcher.think(board, depth);
                    let elapsed = start_time.elapsed();

                    info!("Search run for {} seconds", elapsed.as_secs_f64());
                }
                ActionMessage::SetOption(option_typ) => searcher.handle_set_option(option_typ),
            },
            Err(_) => {
                info!("Disconnected from main thread");
                break;
            }
        };
    }
}
