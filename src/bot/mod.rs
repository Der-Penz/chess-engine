mod calculation_thread;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender, TryRecvError},
        Arc,
    },
    thread,
};

use calculation_thread::thread_loop;
use evaluation::evaluate_board;
use log::info;
use search::{AbortFlag, Search};

use crate::game::{Board, Move};

mod evaluation;
pub mod search;

pub enum ActionMessage {
    Think(Board, u8),
}

pub enum ReactionMessage {
    BestMove(Move),
    Info(String),
}

pub struct Bot {
    board: Board,
    abort_flag: AbortFlag,
    action_sender: Sender<ActionMessage>,
    reaction_receiver: Receiver<ReactionMessage>,
    _thread_handle: thread::JoinHandle<()>,
}

impl Bot {
    pub fn new<S: Search + Send + 'static>(search: S) -> Bot {
        let (action_tx, action_rx) = std::sync::mpsc::channel();
        let (reaction_tx, reaction_rx) = std::sync::mpsc::channel();
        let abort_flag = Arc::new(AtomicBool::new(false));

        let flag = Arc::clone(&abort_flag);
        let calculation_thread =
            thread::spawn(move || thread_loop(action_rx, reaction_tx, flag, search));

        Bot {
            board: Board::default(),
            abort_flag: abort_flag,
            action_sender: action_tx,
            reaction_receiver: reaction_rx,
            _thread_handle: calculation_thread,
        }
    }

    pub fn set_board(&mut self, board: Board) {
        self.board = board;
    }

    pub fn get_board(&self) -> &Board {
        &self.board
    }

    pub fn eval_board(&self) -> i64 {
        evaluate_board(&self.board, None)
    }

    pub fn poll_reaction(&self) -> Result<ReactionMessage, TryRecvError> {
        self.reaction_receiver.try_recv()
    }

    pub fn think(&mut self, depth: u8) {
        info!("Starting search with depth {}", depth);

        self.abort_flag.store(false, Ordering::Relaxed);

        self.action_sender
            .send(ActionMessage::Think(self.board.clone(), depth))
            .unwrap();
    }

    pub fn perft(&self, _depth: u8) {
        todo!("implement perft with new code structure");
        // info!("Starting blocking perft with depth {}", depth);
        // let result = perft(depth, &mut self.board.clone(), false);

        // let mut total = 0;
        // for (mv, counter) in result {
        //     total += counter.count;
        //     println!("{} : {}", mv.as_uci_notation(), counter.count);
        // }

        // println!("\nTotal nodes: {}", total);
    }

    pub fn stop(&mut self) {
        self.abort_flag.store(true, Ordering::Relaxed);
    }

    // pub fn is_running(&self) -> bool {
    //     self.abort_flag.load(Ordering::Relaxed)
    // }
}
