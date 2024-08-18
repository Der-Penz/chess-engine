use std::{
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use log::info;
use search::Search;

use crate::game::{Board, Move};

mod evaluation;
pub mod search;

pub enum Message {
    BestMoveUpdate(Move),
    Info(String),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AbortFlagState {
    Running,
    Stopped,
}

type AbortFlag = Arc<Mutex<AbortFlagState>>;

pub struct Bot {
    board: Board,
    search: Box<dyn search::Search>,
    best_move: Option<Move>,
    abort_flag: AbortFlag,
    // channel_receiver: Receiver<Message>,
    // channel_sender: Sender<Message>,
}

impl Bot {
    pub fn new(search: Box<dyn search::Search>) -> Bot {
        let flag = Arc::new(Mutex::new(AbortFlagState::Stopped));
        Bot {
            board: Board::default(),
            search,
            best_move: None,
            abort_flag: flag,
        }
    }

    pub fn set_board(&mut self, board: Board) {
        self.board = board;
    }

    pub fn get_board(&self) -> &Board {
        &self.board
    }

    pub fn run(&mut self, depth: u8) {
        // let mut best_move = None;
        let abort_flag = Arc::clone(&self.abort_flag);
        let board = self.board.clone();

        let _handle = thread::spawn(move || {
            let mut flag = abort_flag.lock().unwrap();
            *flag = AbortFlagState::Running;
            drop(flag);
            let search = search::min_max::MinMaxSearch {};
            let best_move = search.search(board, depth, &abort_flag);
            let mut flag = abort_flag.lock().unwrap();
            if *flag == AbortFlagState::Running {
                *flag = AbortFlagState::Stopped;
                if let Some(mv) = best_move {
                    println!("bestmove {}", mv.as_uci_notation());
                } else {
                    info!("No best move found");
                }
            }
        });
    }

    pub fn get_best_move(&self) -> Option<Move> {
        self.best_move
    }

    pub fn stop(&mut self) {
        let mut flag = self.abort_flag.lock().unwrap();
        *flag = AbortFlagState::Stopped;
    }

    pub fn is_running(&self) -> bool {
        let flag = self.abort_flag.lock().unwrap();
        *flag == AbortFlagState::Running
    }
}
