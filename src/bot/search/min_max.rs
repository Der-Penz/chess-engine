use log::{info, warn};

use crate::{
    bot::{evaluation::evaluate_board, AbortFlag, AbortFlagState},
    game::{board::move_gen::MoveGeneration, Board, Move},
};

use super::Search;

pub struct MinMaxSearch {
    depth: u8,
    flag: AbortFlag,
    best_move: Option<Move>,
}

impl Search for MinMaxSearch {
    fn search(&mut self, mut board: Board, depth: u8, flag: &mut AbortFlag) -> Option<Move> {
        self.flag = flag.clone();
        self.depth = depth;
        self.best_move = None;
        let player = board.side_to_move().perspective() as i64;
        let (alpha, beta) = match player.signum() {
            1 => (i64::MIN, i64::MAX),
            -1 => (i64::MAX, i64::MIN),
            _ => unreachable!(),
        };
        let score = self.nega_max(depth, &mut board, player, alpha, beta);

        if let Some(mv) = self.best_move.as_ref() {
            info!("Move: {:?} ({})", mv, score);
        } else {
            warn!("No best move found");
        }
        self.best_move
    }
}

impl MinMaxSearch {
    pub fn new() -> Self {
        Self {
            depth: 0,
            best_move: None,
            flag: AbortFlag::default(),
        }
    }

    fn nega_max(
        &mut self,
        depth: u8,
        board: &mut Board,
        player: i64,
        alpha: i64,
        beta: i64,
    ) -> i64 {
        {
            let flag = self.flag.lock().unwrap();
            if *flag == AbortFlagState::Stopped {
                info!("Aborting search at depth {}", self.depth - depth);
                return i64::MIN;
            }
        }

        let move_list = MoveGeneration::generate_legal_moves(board);

        if depth == 0 {
            return evaluate_board(board, Some(move_list)).abs() * player;
        }

        let mut max_score = i64::MIN;

        if move_list.is_empty() {
            // checkmate or stalemate
            return match board.in_check() {
                true => i64::MIN + 1,
                false => -10, // stalemate is bad for the side to move
            };
        }
        for mv in move_list.iter() {
            board.make_move(mv, true, false).unwrap();

            let score = self
                .nega_max(
                    depth - 1,
                    board,
                    -player,
                    beta.saturating_neg(),
                    alpha.saturating_neg(),
                )
                .saturating_neg();
            board.undo_move(mv, true).unwrap();

            let alpha = alpha.max(score);
            if alpha >= beta {
                info!("Cutoff at depth {}", self.depth - depth);
                break;
            }

            if score > max_score {
                max_score = score;
                if depth == self.depth {
                    self.best_move = Some(*mv);
                }
            }
        }

        max_score
    }
}
