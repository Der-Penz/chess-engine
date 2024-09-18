pub(crate) mod calculation_thread;
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
use search::AbortFlag;

use crate::{
    game::{Board, GameResult, Move, MoveGeneration},
    perft,
    uci::commands::command_set_option::OptionType,
};

mod evaluation;
pub mod search;

pub enum ActionMessage {
    Think(Board, u8),
    SetOption(OptionType),
}

pub enum ReactionMessage {
    BestMove(Move),
    Info(String),
}

pub(self) const INFINITY_DEPTH: u8 = 50;

pub struct Bot {
    board: Board,
    abort_flag: AbortFlag,
    thinking: bool,
    action_sender: Sender<ActionMessage>,
    reaction_receiver: Receiver<ReactionMessage>,
    _thread_handle: thread::JoinHandle<()>,
}

impl Bot {
    pub fn new() -> Bot {
        let (action_tx, action_rx) = std::sync::mpsc::channel();
        let (reaction_tx, reaction_rx) = std::sync::mpsc::channel();
        let abort_flag = Arc::new(AtomicBool::new(false));

        let flag = Arc::clone(&abort_flag);
        let calculation_thread = thread::spawn(move || thread_loop(action_rx, reaction_tx, flag));

        Bot {
            board: Board::default(),
            abort_flag: abort_flag,
            thinking: false,
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

    pub fn send_message(&self, msg: ActionMessage) {
        self.action_sender.send(msg).unwrap();
    }

    pub fn eval_board(&mut self, divide: bool) -> String {
        let mut eval_str = String::new();
        if divide {
            MoveGeneration::generate_legal_moves(&self.board)
                .iter()
                .for_each(|mv| {
                    let valid = self.board.make_move(mv, true, false);

                    if valid.is_err() {
                        return;
                    }

                    match GameResult::get_game_result(&self.board, None) {
                        result if result.is_draw() => {
                            eval_str.push_str(&format!("{} : DRAW\n", mv.as_uci_notation()))
                        }
                        result if result.color_lost().is_some() => eval_str.push_str(&format!(
                            "{} : MATE({})\n",
                            mv.as_uci_notation(),
                            result.color_lost().unwrap()
                        )),
                        _ => {
                            let eval = -evaluate_board(&self.board, None);
                            eval_str.push_str(&format!("{} : {}\n", mv.as_uci_notation(), eval));
                        }
                    };

                    self.board.undo_move(mv, true).expect("Must be undo able");
                });
            eval_str.push('\n');
        }

        eval_str.push_str(&format!(
            "Current eval: {}",
            evaluate_board(&self.board, None)
        ));

        eval_str
    }

    pub fn poll_reaction(&mut self) -> Result<ReactionMessage, TryRecvError> {
        let msg = self.reaction_receiver.try_recv();

        if msg
            .as_ref()
            .is_ok_and(|m| matches!(m, ReactionMessage::BestMove(_)))
        {
            self.thinking = false;
        }
        msg
    }

    pub fn think(&mut self, depth: u8) {
        if self.thinking {
            warn!("Bot is already thinking, abort search first");
            return;
        }

        self.abort_flag.store(false, Ordering::Relaxed);
        self.thinking = true;

        self.action_sender
            .send(ActionMessage::Think(self.board.clone(), depth))
            .unwrap();
    }

    /// Returns a string with the perft results for the given depth.
    /// This function is blocking
    pub fn perft(&self, depth: u8) -> String {
        let result = perft(depth, &mut self.board.clone(), false);

        let mut total = 0;
        let mut str = String::new();
        for (mv, counter) in result {
            total += counter.count;
            str.push_str(&format!("{} : {}\n", mv.as_uci_notation(), counter.count));
        }

        str.push_str(&format!("\nTotal nodes: {}", total));
        str
    }

    pub fn stop(&mut self) {
        self.abort_flag.store(true, Ordering::Relaxed);
    }

    pub fn is_running(&self) -> bool {
        self.thinking
    }
}
