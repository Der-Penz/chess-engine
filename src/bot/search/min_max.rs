use log::info;

use crate::{
    bot::{evaluation::evaluate_board, AbortFlag, AbortFlagState},
    game::{board::move_gen::MoveGeneration, Board, Move},
};

use super::Search;

pub struct MinMaxSearch {
    depth: u8,
    flag: AbortFlag,
    best: Option<(Move, i64)>,
    aborted: bool,
    count: u64,
}

impl Search for MinMaxSearch {
    fn search(&mut self, mut board: Board, depth: u8, flag: &mut AbortFlag) -> Option<(Move, i64)> {
        self.flag = flag.clone();
        self.depth = depth;
        self.aborted = false;
        self.best = None;

        let color = board.side_to_move().perspective() as i64;
        let (alpha, beta) = match color {
            1 => (i64::MIN, i64::MAX),
            -1 => (i64::MAX, i64::MIN),
            _ => unreachable!(),
        };
        let max_eval = self.nega_max(&mut board, depth, alpha, beta, color);

        if self.best.is_some() {
            assert_eq!(max_eval, self.best.unwrap().1);
        }

        info!("Evaluated {} nodes", self.count);
        self.best
    }
}

impl MinMaxSearch {
    pub fn new() -> Self {
        Self {
            depth: 0,
            best: None,
            aborted: false,
            flag: AbortFlag::default(),
            count: 0,
        }
    }

    fn nega_max(&mut self, board: &mut Board, depth: u8, alpha: i64, beta: i64, color: i64) -> i64 {
        self.count += 1;

        if self.aborted {
            return i64::MIN;
        } else {
            let flag = self.flag.lock().unwrap();
            if *flag == AbortFlagState::Stopped {
                info!("Aborting search at depth {}", self.depth - depth);
                self.aborted = true;
                return i64::MIN;
            }
        }

        let mut move_gen = MoveGeneration::new();
        let move_list = move_gen.generate_legal_moves(board);
        //TODO move ordering
        if depth == 0 {
            return evaluate_board(board, Some(move_list)).abs() * color;
        }

        // checkmate or stalemate
        if move_list.is_empty() {
            return match board.in_check() {
                true => i64::MIN + 1,
                false => -10, // stalemate is bad for the side to move
            };
        }

        let mut max_eval = i64::MIN;
        let mut alpha = alpha;
        for mv in move_list.iter() {
            board.make_move(mv, true, false).unwrap();

            let eval = self
                .nega_max(
                    board,
                    depth - 1,
                    beta.saturating_neg(),
                    alpha.saturating_neg(),
                    -color,
                )
                .saturating_neg();

            board.undo_move(mv, true).unwrap();

            if eval > max_eval {
                max_eval = eval;

                if depth == self.depth {
                    info!("New best move: {:?} with eval {}", mv, eval);
                    self.best = Some((*mv, eval));
                }
            }

            alpha = alpha.max(max_eval);
            if alpha >= beta {
                break;
            }
        }

        max_eval
    }
}
