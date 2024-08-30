use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::{
    bot::{evaluation::evaluate_board, AbortFlag},
    game::{board::move_gen::MoveGeneration, Board, Move},
};

use super::Search;

pub struct MinMaxSearch {
    depth: u8,
    flag: AbortFlag,
    best: Option<(Move, i64)>,
    aborted: bool,
    node_count: u64,
    board: Board,
}

impl Search for MinMaxSearch {
    fn search(
        &mut self,
        board: Board,
        depth: u8,
        flag: &AbortFlag,
        _msg_channel: &mut std::sync::mpsc::Sender<crate::bot::ReactionMessage>,
    ) -> Option<(Move, i64)> {
        self.flag = Arc::clone(flag);
        self.depth = depth;
        self.aborted = false;
        self.best = None;
        self.board = board;

        let best_score = self.nega_max(depth, 0, NEG_INF, POS_INF);

        info!("Nodes searched: {}", self.node_count);
        info!("Score: {}", best_score);
        return self.best;
    }
}

const NEG_INF: i64 = i64::MIN;
const POS_INF: i64 = i64::MAX;

const DRAW: i64 = 0;
const MATE: i64 = 10000;

impl MinMaxSearch {
    pub fn new() -> Self {
        Self {
            depth: 0,
            best: None,
            board: Board::default(),
            aborted: false,
            flag: Arc::new(AtomicBool::new(false)),
            node_count: 0,
        }
    }

    #[inline(always)]
    /// Check if the search has been cancelled
    /// and set the aborted flag if it has to make it faster instead of checking the
    /// atomic bool every time
    fn search_cancelled(&mut self) -> bool {
        if self.aborted {
            return true;
        } else {
            if self.flag.load(Ordering::Relaxed) {
                info!("Search aborted");
                self.aborted = true;
                return true;
            }
        }
        false
    }

    fn nega_max(&mut self, ply_remaining: u8, ply_from_root: u8, mut alpha: i64, beta: i64) -> i64 {
        //check if the search has been aborted
        if self.search_cancelled() {
            return 0;
        }

        self.node_count += 1; //increment node count
        let moves = MoveGeneration::generate_legal_moves(&self.board);

        if ply_remaining == 0 {
            return evaluate_board(&self.board, Some(&moves));
        }

        //check for checkmate and stalemate
        if moves.is_empty() {
            if moves.get_masks().in_check {
                info!("Checkmate found at depth {}", ply_from_root);
                return -(MATE - ply_from_root as i64);
            } else {
                return DRAW;
            }
        }

        for mov in moves.iter() {
            self.board.make_move(mov, true, false).unwrap();

            let new_alpha = beta.saturating_neg();
            let new_beta = alpha.saturating_neg();
            let eval = self
                .nega_max(ply_remaining - 1, ply_from_root + 1, new_alpha, new_beta)
                .saturating_neg();

            self.board.undo_move(mov, true).unwrap();

            if self.search_cancelled() {
                return 0;
            }

            if eval >= beta {
                return beta;
            }

            if eval > alpha {
                alpha = eval;
                if ply_from_root == 0 {
                    self.best = Some((*mov, eval));
                }
            }
        }

        alpha
    }
}
