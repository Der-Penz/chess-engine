use crate::{
    bot::{
        evaluation::{eval::*, evaluate_board},
        AbortFlag, ReactionMessage, INFINITY_DEPTH,
    },
    game::{board::move_gen::MoveGeneration, Board, Move},
    uci::commands::command_set_option::OptionType,
};
use std::sync::{atomic::Ordering, mpsc::Sender};

use super::{
    diagnostics::SearchDiagnostics,
    limit::Limits,
    move_ordering::MoveOrdering,
    opening_book::OpeningBook,
    pv_line::PVLine,
    repetition_history::RepetitionHistory,
    transposition_table::{NodeType, TranspositionTable},
};

pub const MAX_QS_DEPTH: u8 = 8;

pub struct Searcher {
    best: Option<(Move, Eval)>,
    aborted: bool,
    board: Board,
    flag: AbortFlag,
    msg_channel: Sender<ReactionMessage>,
    diagnostics: SearchDiagnostics,
    tt: TranspositionTable,
    pv_line: PVLine,
    opening_book: Option<OpeningBook>,
    repetition_history: RepetitionHistory,
    limits: Limits,
}

impl Searcher {
    pub fn new(
        tt: TranspositionTable,
        msg_channel: Sender<ReactionMessage>,
        flag: AbortFlag,
        opening_book_file: String,
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
            opening_book: OpeningBook::new(opening_book_file).ok(),
            repetition_history: RepetitionHistory::new(),
            limits: Limits::new(),
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
            OptionType::Threads(_) => {
                todo!("Threads option not implemented yet");
            }
            OptionType::DebugFile(_) => {
                todo!("Debug file option not implemented yet");
            }
            OptionType::OwnBook(value) => {
                if let Some(book) = &mut self.opening_book {
                    book.set_enabled(value);
                    info!("Opening book enabled: {value}");
                } else {
                    warn!("No opening book loaded, cannot enable or disable it");
                }
            }
        }
    }

    pub fn think(&mut self, board: Board, limits: Limits) {
        self.aborted = false;
        self.best = None;
        self.board = board;
        self.diagnostics.reset();
        self.pv_line.reset();
        self.repetition_history.init(&self.board);
        self.limits = limits;

        info!(
            "Transposition Table Usage: {:.2}%",
            self.tt.get_usage() * 100_f64
        );

        // Check if we have a move in the opening book
        if let Some(opening_book) = &self.opening_book {
            if let Some(mv) = opening_book.get_random_book_move(&self.board) {
                info!("Play opening book move: {:?}", mv);
                self.msg_channel
                    .send(ReactionMessage::BestMove(mv))
                    .unwrap();
                return;
            }
        }

        self.iterative_deepening();
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

    fn iterative_deepening(&mut self) {
        for depth in 1..=INFINITY_DEPTH {
            let now = std::time::Instant::now();
            let _ = self.nega_max(depth, 0, NEG_INF, POS_INF);
            let elapsed = now.elapsed().as_millis();
            info!("Iterative Deepening depth {} done", depth);

            let best_this_iteration = self.best;

            self.try_build_pv_line(depth);

            if let Some((_, eval)) = best_this_iteration {
                self.send_info(format!(
                    "depth {} score {} nodes {:?} time {} hashfull {:.2} pv {}",
                    depth,
                    display_eval(eval),
                    self.diagnostics.node_count,
                    elapsed,
                    self.tt.get_usage() * 100_f64,
                    self.pv_line
                ));

                //close the search if a mate score is found
                //TODO maybe add a config option to continue the search if a mate score is found
                // if is_mate_score(score) {
                //     info!("Mate score found, stopping search");
                //     break;
                // }
            }

            if self
                .limits
                .is_any_terminal(self.diagnostics.node_count, depth)
            {
                info!("Stopping");
                break;
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

        //two fold repetition rule
        if self.repetition_history.two_fold_repetition(key)
            || self.board.cur_state().ply_clock >= 100
        {
            return DRAW;
        }

        let mut tt_move = None;
        if let Some(entry) = self.tt.get_entry(key, ply_remaining) {
            self.diagnostics.inc_tt_hits();
            tt_move = entry.best_move;

            let mut eval = entry.eval;
            TranspositionTable::correct_retrieved_mate_score(&mut eval, ply_from_root);

            match entry.node_type {
                NodeType::Exact => {
                    if ply_from_root == 0 {
                        if let Some(mv) = entry.best_move {
                            if !self.best.is_some_and(|(_, e)| e > eval) {
                                self.best = Some((mv, eval));
                            }
                        }
                    }
                    // return eval;
                }
                NodeType::LowerBound => {
                    alpha = alpha.max(eval);
                }
                NodeType::UpperBound => {
                    beta = beta.min(eval);
                }
            }
            if alpha >= beta {
                self.diagnostics.inc_cut_offs();
                return eval;
            }
        }

        if self
            .limits
            .is_any_terminal(self.diagnostics.node_count, ply_from_root)
            || ply_remaining == 0
        {
            return self.quiescence_search(ply_from_root, MAX_QS_DEPTH, alpha, beta);
        }

        let moves = MoveGeneration::generate_legal_moves(&self.board);
        if moves.is_checkmate() {
            return -(MATE - ply_from_root as Eval);
        }
        if moves.is_stalemate() {
            return DRAW;
        }

        let mut ordered_moves =
            MoveOrdering::score_moves(&moves, &self.pv_line, ply_from_root, &self.board, tt_move);

        let mut best_move_this_position = None;
        let mut node_type = NodeType::UpperBound;

        while let Some(mov) = ordered_moves.pick_next_move() {
            self.repetition_history.push_hash(key, false);
            self.board.make_move(&mov, true, false).unwrap();

            let eval = -self.nega_max(ply_remaining - 1, ply_from_root + 1, -beta, -alpha);

            self.repetition_history.pop_hash();
            self.board.undo_move(&mov, true).unwrap();

            if self.search_cancelled() {
                return 0;
            }

            if eval >= beta {
                self.diagnostics.inc_cut_offs();
                self.tt.insert(
                    key,
                    ply_remaining,
                    ply_from_root,
                    beta,
                    NodeType::LowerBound,
                    Some(mov),
                );
                return beta;
            }

            if eval > alpha {
                alpha = eval;

                best_move_this_position = Some(mov);
                node_type = NodeType::Exact;

                if ply_from_root == 0 {
                    self.best = Some((mov, eval));
                }
            }
        }

        self.tt.insert(
            key,
            ply_remaining,
            ply_from_root,
            alpha,
            node_type,
            best_move_this_position,
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
    fn quiescence_search(
        &mut self,
        ply_from_root: u8,
        ply_remaining: u8,
        mut alpha: Eval,
        beta: Eval,
    ) -> Eval {
        //TODO maybe add forced moves to the quiescence search to see checkmates faster e.g:r4r1k/1R1R2p1/7p/8/8/3Q1Ppq/P7/6K1 w - - 0 1 king is forced to move
        //which could already be seen at depth 3 or lower
        if self.search_cancelled() {
            return 0;
        }
        self.diagnostics.inc_node_qs();

        let key = self.board.cur_state().zobrist;
        //two fold repetition rule
        if self.repetition_history.two_fold_repetition(key)
            || self.board.cur_state().ply_clock >= 100
        {
            return DRAW;
        }

        let moves = MoveGeneration::generate_legal_moves(&self.board);
        if moves.is_checkmate() {
            return -(MATE - ply_from_root as Eval);
        }
        if moves.is_stalemate() {
            return DRAW;
        }

        let mut eval = evaluate_board(&self.board, Some(&moves));
        if eval >= beta {
            return beta;
        }
        if eval > alpha {
            alpha = eval;
        }

        if ply_remaining == 0 {
            return eval;
        }

        let moves = moves.to_captures_only();
        let mut move_order = MoveOrdering::score_moves(
            &moves,
            &self.pv_line,
            self.pv_line.len() as u8,
            &self.board,
            None,
        );
        while let Some(mv) = move_order.pick_next_move() {
            self.repetition_history.push_hash(key, false);
            self.board.make_move(&mv, true, false).unwrap();

            eval = -self.quiescence_search(ply_from_root + 1, ply_remaining - 1, -beta, -alpha);

            self.repetition_history.pop_hash();
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
