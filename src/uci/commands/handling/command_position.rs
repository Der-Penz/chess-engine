use itertools::Itertools;
use log::{error, info};

use crate::{
    bot::Bot,
    game::{move_notation::Move, Board},
    uci::commands::{CommandParseError, UCICommand},
};

#[derive(Debug, PartialEq)]
pub struct PositionParams {
    pub fen: Option<String>,
    pub moves: Vec<Move>,
}

pub fn handle_position(bot: &mut Bot, params: PositionParams) -> Option<String> {
    let mut board = match params.fen {
        Some(fen) => match Board::from_fen(&fen) {
            Ok(b) => b,
            Err(err) => {
                error!("Error parsing FEN: {}", err);
                return Some("quit".into());
            }
        },
        None => Board::default(),
    };

    params.moves.iter().for_each(|m| {
        board
            .make_move(m, false, true)
            .expect("UCI received invalid move that cannot be played by the engine");
    });

    info!("Position set to: {}", board.to_fen());
    bot.set_board(board);
    None
}

pub fn parse_position(params: &str) -> Result<UCICommand, CommandParseError> {
    let mut parts = params.split_whitespace();
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
            return Err(CommandParseError::ParseError(
                "Missing \"FEN\" or \"startpos\" literal".into(),
            ));
        }
    };

    let mut board = if let Some(fen) = &fen {
        Board::from_fen(fen)
            .map_err(|_| CommandParseError::ParseError("Invalid FEN position".to_string()))?
    } else {
        Board::default()
    };
    let moves = if parts.next() == Some("moves") {
        let mut moves = vec![];
        for mov_str in parts {
            let mov = Move::from_uci_notation(mov_str, &board).map_err(|_| {
                CommandParseError::ParseError(format!("Move {} is not valid uci notation", mov_str))
            })?;

            board.make_move(&mov, false, true).map_err(|_| {
                CommandParseError::ParseError(format!(
                    "Move {:?} is invalid for the position {}",
                    mov,
                    board.to_fen()
                ))
            })?;
            moves.push(mov);
        }
        moves
    } else {
        Vec::new()
    };

    Ok(UCICommand::Position(PositionParams { fen, moves }))
}
