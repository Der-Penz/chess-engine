use std::sync::mpsc::{Receiver, Sender};

use super::{
    search::{
        searcher::Searcher,
        transposition_table::{TranspositionTable, DEFAULT_HASH_SIZE},
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
    let tt = TranspositionTable::new(DEFAULT_HASH_SIZE);

    let opening_book_path = std::env::var("OPENING_BOOK").unwrap_or("".to_string());
    let mut searcher = Searcher::new(tt, tx, flag, opening_book_path);

    loop {
        match receiver.recv() {
            Ok(value) => match value {
                ActionMessage::Think(board, limits) => {
                    let start_time = std::time::Instant::now();
                    searcher.think(board, limits);
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
