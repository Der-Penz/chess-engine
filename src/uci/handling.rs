use crate::game::{Board, Move};
use super::commands::Command;
use log::error;
use rand::seq::SliceRandom;

pub fn handle_uci_command(command: Command, board: &mut Board) -> Option<String> {
    match command {
        Command::UCINewGame => None,
        Command::Quit => None,
        Command::UCI => Some(handle_setup().to_string()),
        Command::IsReady => Some("readyok".to_string()),
        Command::SetOption(_, _) => None,
        Command::Position(pos, moves) => handle_position(board, &pos, &moves),
        Command::Display => Some(board.to_string()),
        Command::Go => handle_go(board),
    }
}

fn handle_position(board: &mut Board, pos: &Option<String>, moves: &Vec<Move>) -> Option<String> {
    *board = match pos {
        Some(fen) => {
            match Board::from_fen(&fen) {
                Ok(b) => b,
                Err(err) => {
                    error!("Error parsing FEN: {}", err);
                    return Some("quit".into());
                }
            }
        }
        None => Board::base(),
    };

    for m in moves {
        board.move_piece(m.source(), m.dest());
    }

    None
}

fn handle_go(board: &mut Board) -> Option<String> {
    let moves = board.get_all_possible_moves();
    let best_move = moves.choose(&mut rand::thread_rng());

    match best_move {
        Some(m) if m.valid() => {
            board.move_piece(m.source(), m.dest());
            Some(format!("bestmove {}", m.to_long_algebraic().expect("Invalid move")))
        }
        _ => None,
    }
}

const AUTHOR: &str = "DerPenz";
const NAME: &str = "Chesse";

fn handle_setup() -> String {
    format!("id name {}\n id author {}\nuciok", NAME, AUTHOR)
}
