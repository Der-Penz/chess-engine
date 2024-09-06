use crate::{
    bot::{
        evaluation::{eval::*, evaluate_board},
        AbortFlag, ReactionMessage,
    },
    game::{board::move_gen::MoveGeneration, Board, Move},
    uci::commands::command_set_option::OptionType,
};
use std::sync::{atomic::Ordering, mpsc::Sender};

use super::{
    diagnostics::SearchDiagnostics,
    move_ordering::MoveOrdering,
    pv_line::PVLine,
    transposition_table::{NodeType, TranspositionTable, TranspositionTableEntry},
};

pub struct Searcher {
    best: Option<(Move, Eval)>,
    aborted: bool,
    board: Board,
    flag: AbortFlag,
    msg_channel: Sender<ReactionMessage>,
    diagnostics: SearchDiagnostics,
    tt: TranspositionTable,
    pv_line: PVLine,
    max_qs_depth: u8,
}

impl Searcher {
    pub fn new(
        tt: TranspositionTable,
        msg_channel: Sender<ReactionMessage>,
        flag: AbortFlag,
    ) -> Self {
        Self {
            best: None,
            board: Board::default(),
            aborted: false,
            diagnostics: SearchDiagnostics::default(),
            msg_channel,
            flag,
            tt,
            pv_line: PVLine::default(),
            max_qs_depth: 8,
        }
    }

    pub fn handle_set_option(&mut self, option: OptionType) {
        match option {
            OptionType::ClearHash => {
                self.tt.clear();
                info!("Transposition Table cleared");
            }
            OptionType::HashSize(size) => {
                self.tt.set_size(size);
                info!("Transposition Table size set to {size}mb");
            }
        }
    }

    pub fn think(&mut self, board: Board, depth: u8) {
        self.aborted = false;
        self.best = None;
        self.board = board;
        self.diagnostics.reset();

        info!(
            "Transposition Table Usage: {:.2}%",
            self.tt.get_usage() * 100_f64
        );

        self.iterative_deepening(depth);
        info!("Search Diagnostics: {}", self.diagnostics);

        let result = self.best.unwrap_or_else(|| {
            warn!("No best move found, using any random move");
            let moves = MoveGeneration::generate_legal_moves(&self.board);

            moves.get(0).map(|m| (m, 0)).unwrap_or_default()
        });

        self.msg_channel
            .send(ReactionMessage::BestMove(result.0))
            .unwrap();
    }

    #[inline(always)]
    /// Check if the search has been cancelled
    /// and set the aborted flag if it has to make it faster instead of checking the
    /// atomic bool every time
    fn search_cancelled(&mut self) -> bool {
        if self.aborted {
            return true;
        }
        if self.flag.load(Ordering::Relaxed) {
            info!("Search aborted");
            self.aborted = true;
            return true;
        }

        false
    }

    fn send_info(&self, msg: String) {
        self.msg_channel.send(ReactionMessage::Info(msg)).ok();
    }

    fn iterative_deepening(&mut self, depth: u8) {
        self.pv_line.reset();

        for depth in 1..=depth {
            let now = std::time::Instant::now();
            let eval = self.nega_max(depth, 0, NEG_INF, POS_INF);
            info!("Iterative Deepening depth {} done", depth);

            let best_this_iteration = self.best;

            self.try_build_pv_line(depth);

            if let Some((_, score)) = best_this_iteration {
                assert_eq!(score, eval);
                self.send_info(format!(
                    "depth {} score cp {} nodes {:?} time {} hashfull {:.2} pv {}",
                    depth,
                    score,
                    self.diagnostics.node_count,
                    now.elapsed().as_millis(),
                    self.tt.get_usage() * 100_f64,
                    self.pv_line
                ));

                //close the search if a mate score is found
                //TODO maybe add a config option to continue the search if a mate score is found
                if is_mate_score(score) {
                    info!("Mate score found, stopping search");
                    break;
                }
            }

            if self.search_cancelled() {
                break;
            }
        }
    }

    fn try_build_pv_line(&mut self, depth: u8) {
        self.pv_line.reset();
        for i in 0..depth as usize {
            let entry = match self
                .tt
                .get_entry(self.board.cur_state().zobrist, depth - i as u8)
            {
                Some(entry) => entry,
                None => break,
            };

            let mv = match entry.best_move {
                Some(mv) => mv,
                None => break,
            };
            let valid = self.board.make_move(&mv, true, true);

            if valid.is_err() {
                warn!("Stopping PVLine due to invalid move in TT entry: {}", mv);
                break;
            }

            self.pv_line.add(mv);
        }

        for i in 0..self.pv_line.len() {
            let idx: usize = self.pv_line.len() - i - 1;
            self.board
                .undo_move(&self.pv_line.get_move(idx).unwrap(), true)
                .unwrap();
        }
    }

    fn nega_max(
        &mut self,
        ply_remaining: u8,
        ply_from_root: u8,
        mut alpha: Eval,
        mut beta: Eval,
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
        let mut tt_move = None;
        if let Some(entry) = self.tt.get_entry(key, ply_remaining) {
            self.diagnostics.inc_tt_hits();
            tt_move = entry.best_move;

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

        if ply_remaining == 0 {
            return self.quiescence_search(alpha, beta, self.max_qs_depth);
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

        let mut ordered_moves =
            MoveOrdering::score_moves(&moves, &self.pv_line, ply_from_root, &self.board, tt_move);

        let mut best_move_this_position = None;

        while let Some(mov) = ordered_moves.pick_next_move() {
            self.board.make_move(&mov, true, false).unwrap();

            let eval = -self.nega_max(ply_remaining - 1, ply_from_root + 1, -beta, -alpha);

            self.board.undo_move(&mov, true).unwrap();

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
                        best_move: Some(mov),
                    },
                    false,
                );
                return beta;
            }

            if eval > alpha {
                alpha = eval;

                best_move_this_position = Some(mov);

                if ply_from_root == 0 {
                    self.best = Some((mov, eval));
                }
            }
        }

        let node_type = NodeType::type_from_eval(alpha, original_alpha, beta);
        let is_pv = matches!(node_type, NodeType::Exact);
        self.tt.insert(
            key,
            TranspositionTableEntry {
                zobrist: key,
                depth: ply_remaining,
                eval: alpha,
                node_type,
                best_move: best_move_this_position,
            },
            is_pv,
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
    fn quiescence_search(&mut self, mut alpha: Eval, beta: Eval, depth: u8) -> Eval {
        if self.search_cancelled() {
            return 0;
        }
        self.diagnostics.inc_node_qs();

        //evaluate the position and check if we can prune it early
        let moves = MoveGeneration::generate_legal_moves_captures(&self.board);
        let mut eval = evaluate_board(&self.board, Some(&moves));
        if eval >= beta {
            return beta;
        }
        if eval > alpha {
            alpha = eval;
        }

        if depth == 0 {
            return eval;
        }

        let mut move_order = MoveOrdering::score_moves(
            &moves,
            &self.pv_line,
            self.pv_line.len() as u8,
            &self.board,
            None,
        );
        while let Some(mv) = move_order.pick_next_move() {
            self.board.make_move(&mv, true, false).unwrap();

            eval = -self.quiescence_search(-beta, -alpha, depth - 1);

            self.board.undo_move(&mv, true).unwrap();

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
