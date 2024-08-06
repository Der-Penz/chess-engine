use itertools::Itertools;
use log::{error, info};

use crate::{
    game::{move_notation::Move, Board},
    uci::commands::{Command, CommandParseError},
};

pub fn handle_position(
    board: &mut Board,
    pos: &Option<String>,
    moves: &Vec<Move>,
) -> Option<String> {
    *board = match pos {
        Some(fen) => match Board::from_fen(&fen) {
            Ok(b) => b,
            Err(err) => {
                error!("Error parsing FEN: {}", err);
                return Some("quit".into());
            }
        },
        None => Board::default(),
    };

    moves.iter().for_each(|m| {
        board
            .make_move(m, false, true)
            .expect("UCI received invalid move that cannot be played by the engine");
        // .expect("UCI received invalid move that cannot be played by the engine");
    });
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

    let mut board = Board::default();
    if parts.next() == Some("moves") {
        let moves: Vec<Move> = parts
            .map(|s: &str| {
                let mov = Move::from_uci_notation(s, &board)
                    .expect("Should be a parsable move from uci gui");

                board
                    .make_move(&mov, false, true)
                    .expect("Move should be valid");
                return mov;
            })
            .collect();

        Ok(Command::Position(fen, moves))
    } else {
        Ok(Command::Position(fen, Vec::new()))
    }
}
