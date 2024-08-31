use crate::{
    bot::{
        evaluation::{eval::*, evaluate_board},
        AbortFlag, ReactionMessage,
    },
    game::{board::move_gen::MoveGeneration, Board, Move},
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
    Arc,
};

use super::{diagnostics::SearchDiagnostics, Search};

pub struct MinMaxSearch {
    depth: u8,
    flag: Option<AbortFlag>,
    best: Option<(Move, Eval)>,
    aborted: bool,
    board: Board,
    msg_channel: Option<Sender<ReactionMessage>>,
    diagnostics: SearchDiagnostics,
}

impl Search for MinMaxSearch {
    fn set_communication_channels(
        &mut self,
        abort_flag: Arc<AtomicBool>,
        msg_channel: std::sync::mpsc::Sender<crate::bot::ReactionMessage>,
    ) {
        self.flag = Some(abort_flag);
        self.msg_channel = Some(msg_channel);
    }

    fn search(&mut self, board: Board, depth: u8) -> Option<(Move, Eval)> {
        self.depth = depth;
        self.aborted = false;
        self.best = None;
        self.board = board;

        if self.flag.is_none() {
            warn!("Abort flag not set, search will not be cancellable");
        }

        let best_score = self.nega_max(depth, 0, NEG_INF, POS_INF);

        info!("Search Diagnostics: {}", self.diagnostics);
        info!("Score: {}", best_score);
        return self.best;
    }
}

impl MinMaxSearch {
    pub fn new() -> Self {
        Self {
            depth: 0,
            best: None,
            board: Board::default(),
            aborted: false,
            flag: None,
            diagnostics: SearchDiagnostics::default(),
            msg_channel: None,
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
            if let Some(flag) = &self.flag {
                if flag.load(Ordering::Relaxed) {
                    info!("Search aborted");
                    self.aborted = true;
                    return true;
                }
            }
        }
        false
    }

    fn nega_max(
        &mut self,
        ply_remaining: u8,
        ply_from_root: u8,
        mut alpha: Eval,
        beta: Eval,
    ) -> Eval {
        //check if the search has been aborted
        if self.search_cancelled() {
            return 0;
        }
        self.diagnostics.inc_node();

        let moves = MoveGeneration::generate_legal_moves(&self.board);

        if ply_remaining == 0 {
            let eval = self.quiescence_search(alpha, beta);
            // return evaluate_board(&self.board, Some(&moves));
            return eval;
        }

        //check for checkmate and stalemate
        if moves.is_empty() {
            if moves.get_masks().in_check {
                info!("Checkmate found at depth {}", ply_from_root);
                return -(MATE - ply_from_root as Eval);
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
                self.diagnostics.inc_cut_offs();
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

    fn quiescence_search(&mut self, mut alpha: Eval, beta: Eval) -> Eval {
        if self.search_cancelled() {
            return 0;
        }
        self.diagnostics.inc_node_qs();

        let mut eval = evaluate_board(&self.board, None);

        if eval >= beta {
            return beta;
        }
        if eval > alpha {
            alpha = eval;
        }

        let moves = MoveGeneration::generate_legal_moves_captures(&self.board);

        for mv in moves.iter() {
            self.board.make_move(mv, true, false).unwrap();

            eval = -self.quiescence_search(-beta, -alpha);

            self.board.undo_move(mv, true).unwrap();

            if eval >= beta {
                self.diagnostics.inc_cut_offs();
                return beta;
            }
            if eval > alpha {
                alpha = eval;
            }
        }

        return alpha;
    }
}
