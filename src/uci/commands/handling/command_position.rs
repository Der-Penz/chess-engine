use itertools::Itertools;
use log::error;

use crate::{ game::{ Board, Move }, uci::commands::{ Command, CommandParseError } };

pub fn handle_position(
    board: &mut Board,
    pos: &Option<String>,
    moves: &Vec<Move>
) -> Option<String> {
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

    moves.iter().for_each(|m| board.play(m));
    None
}

pub fn parse_position(str: &str) -> Result<Command, CommandParseError> {
    let mut parts = str.split_whitespace();
    parts.next();
    let fen = match parts.next() {
        Some("startpos") => None,
        Some("fen") => {
            let mut iter = parts
                .by_ref()
                .take_while(|r| *r != "moves")
                .map(String::from);
            Some(Itertools::join(&mut iter, " "))
        }
        _ => {
            return Err(CommandParseError::MissingParameter("position".to_string()));
        }
    };

    if parts.next() == Some("moves") {
        let moves = parts.map(|s| Move::from_source_dest(s)).collect_vec();
        Ok(Command::Position(fen, moves))
    } else {
        Ok(Command::Position(fen, Vec::new()))
    }
}
