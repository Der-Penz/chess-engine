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

use super::{
    diagnostics::SearchDiagnostics,
    pv_line::PVLine,
    transposition_table::{
        NodeType, ReplacementStrategy, TranspositionTable, TranspositionTableEntry,
    },
    Search,
};

pub struct MinMaxSearch {
    depth: u8,
    flag: Option<AbortFlag>,
    best: Option<(Move, Eval)>,
    aborted: bool,
    board: Board,
    msg_channel: Option<Sender<ReactionMessage>>,
    diagnostics: SearchDiagnostics,
    tt: TranspositionTable,
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

        info!(
            "Transposition Table Usage: {:.2}%",
            self.tt.get_usage() * 100_f64
        );

        if self.flag.is_none() {
            warn!("Abort flag not set, search will not be cancellable");
        }

        self.iterative_deepening();

        // let best_score = self.nega_max(depth, 0, NEG_INF, POS_INF);

        info!("Search Diagnostics: {}", self.diagnostics);

        self.best.or_else(|| {
            warn!("No best move found, using any random move");
            let moves = MoveGeneration::generate_legal_moves(&self.board);

            moves.get(0).map(|m| (*m, 0))
        })
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
            tt: TranspositionTable::new(20_f64, ReplacementStrategy::ReplaceOnFull(false)),
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

    fn send_info(&self, msg: String) {
        if let Some(channel) = &self.msg_channel {
            channel.send(ReactionMessage::Info(msg)).ok();
        }
    }

    fn iterative_deepening(&mut self) {
        for depth in 1..=self.depth {
            let mut pv_line = PVLine::default();

            self.nega_max(depth, 0, NEG_INF, POS_INF, &mut pv_line);
            let best_this_iteration = self.best;

            if let Some((_, score)) = best_this_iteration.as_ref() {
                self.send_info(format!(
                    "depth {} score cp {} nodes {:?} pv {}",
                    depth, score, self.diagnostics.node_count, pv_line
                ));
            }

            if self.search_cancelled() {
                break;
            }
        }
    }

    fn nega_max(
        &mut self,
        ply_remaining: u8,
        ply_from_root: u8,
        mut alpha: Eval,
        mut beta: Eval,
        pv_line: &mut PVLine,
    ) -> Eval {
        //check if the search has been aborted
        if self.search_cancelled() {
            return 0;
        }
        self.diagnostics.inc_node();
        let key = self.board.cur_state().zobrist;

        //skip the position if it there is a mating sequence earlier in the search which would be shorter than a current one
        alpha = alpha.max(-MATE + ply_from_root as Eval);
        beta = beta.min(MATE - ply_from_root as Eval);
        if alpha >= beta {
            self.diagnostics.inc_cut_offs();
            return alpha;
        }

        let original_alpha = alpha;
        let tt_entry = self.tt.get_entry(key);
        if let Some(entry) = tt_entry {
            if entry.depth >= ply_remaining && entry.zobrist == key {
                self.diagnostics.inc_tt_hits();

                match entry.node_type {
                    NodeType::Exact => {
                        let mut eval = entry.eval;
                        //correct a mate score to be relative to the current position
                        if is_mate_score(eval) {
                            eval = correct_mate_score(eval, ply_from_root);
                        }
                        if ply_from_root == 0 {
                            if let Some(mv) = entry.best_move {
                                if !self.best.is_some_and(|(_, e)| e > eval) {
                                    self.best = Some((mv, eval));
                                }
                            }
                        }
                        return eval;
                    }
                    NodeType::LowerBound => {
                        alpha = alpha.max(entry.eval);
                    }
                    NodeType::UpperBound => {
                        beta = beta.min(entry.eval);
                    }
                }
                if alpha >= beta {
                    self.diagnostics.inc_cut_offs();
                    return entry.eval;
                }
            }
        }

        let mut line = PVLine::default();

        if ply_remaining == 0 {
            line.reset();
            return self.quiescence_search(alpha, beta);
        }

        let moves = MoveGeneration::generate_legal_moves(&self.board);

        //check for checkmate and stalemate
        if moves.is_empty() {
            if moves.get_masks().in_check {
                return -(MATE - ply_from_root as Eval);
            } else {
                return DRAW;
            }
        }

        let mut best_move_this_position = None;

        for mov in moves.iter() {
            self.board.make_move(mov, true, false).unwrap();

            let new_alpha = beta.saturating_neg();
            let new_beta = alpha.saturating_neg();
            let eval = self
                .nega_max(
                    ply_remaining - 1,
                    ply_from_root + 1,
                    new_alpha,
                    new_beta,
                    &mut line,
                )
                .saturating_neg();

            self.board.undo_move(mov, true).unwrap();

            if self.search_cancelled() {
                return 0;
            }

            if eval >= beta {
                self.diagnostics.inc_cut_offs();

                self.tt.insert(
                    key,
                    TranspositionTableEntry {
                        depth: ply_remaining,
                        eval: beta,
                        node_type: NodeType::LowerBound,
                        zobrist: key,
                        best_move: Some(*mov),
                    },
                );
                return beta;
            }

            if eval > alpha {
                alpha = eval;

                best_move_this_position = Some(*mov);

                pv_line.extend_line(*mov, &line);

                if ply_from_root == 0 {
                    self.best = Some((*mov, eval));
                }
            }
        }

        let node_type = if alpha <= original_alpha {
            NodeType::UpperBound
        } else if alpha >= beta {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        };

        self.tt.insert(
            key,
            TranspositionTableEntry {
                zobrist: key,
                depth: ply_remaining,
                eval: alpha,
                node_type,
                best_move: best_move_this_position,
            },
        );

        alpha
    }

    /// Quiescence search is a special search that only searches captures
    /// it helps to avoid the horizon effect where the search would stop at a quiet position
    /// and not see a capture that would change the evaluation drastically
    /// consider the last move being from the queen. Evaluating the position after the queen move
    /// would give a high evaluation, but if the search would look one move further, it could
    /// see that the queen can be captured by a pawn, which would change the evaluation drastically
    /// by only searching captures, we can avoid this problem and get a more accurate evaluation
    /// since only captures are considered, the search is much faster and terminates faster
    fn quiescence_search(&mut self, mut alpha: Eval, beta: Eval) -> Eval {
        if self.search_cancelled() {
            return 0;
        }
        self.diagnostics.inc_node_qs();

        //evaluate the position and check if we can prune it early
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
