use std::{thread, time::Duration};

use log::info;

use crate::{
    bot::{evaluation::evaluate_board, AbortFlag, AbortFlagState, Message},
    game::{board::move_gen::MoveGeneration, Board, Color, Move},
};

use super::Search;

fn min_max(board: &mut Board, depth: u8, color: Color) -> Result<i32, ()> {
    if depth == 0 {
        return Ok(evaluate_board(board));
    }

    let mut best_score = match color {
        Color::White => i32::MIN,
        Color::Black => i32::MAX,
    };

    for mv in MoveGeneration::generate_legal_moves(board).iter() {
        board.make_move(&mv, true, false).map_err(|_| ())?;
        let score = min_max(board, depth - 1, color.opposite())?;
        board.undo_move(mv, true).map_err(|_| ())?;
        best_score = match color {
            Color::White => best_score.max(score),
            Color::Black => best_score.min(score),
        };
    }

    Ok(best_score)
}

pub struct MinMaxSearch {}

impl Search for MinMaxSearch {
    fn search(&self, mut board: Board, depth: u8, flag: &AbortFlag) -> Option<Move> {
        let mut best_move = None;
        let mut best_score = i32::MIN;
        let move_list = MoveGeneration::generate_legal_moves(&board);
        for mv in move_list.iter() {
            let flag = *flag.lock().unwrap();
            if flag == AbortFlagState::Stopped {
                info!("Aborting Search");
                break;
            }

            board.make_move(mv, true, false).unwrap();
            let color = board.side_to_move().opposite();
            let score = min_max(&mut board, depth - 1, color).unwrap();
            board.undo_move(mv, true).unwrap();

            if score > best_score {
                best_score = score;
                best_move = Some(mv);
            }
        }

        best_move.copied()
    }
}
