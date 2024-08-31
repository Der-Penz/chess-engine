use crate::{
    bot::Bot,
    game::{move_notation::Move, Board},
    uci::commands::{CommandParseError, UCICommand},
};

pub struct PositionParams {
    pub board: Board,
}

impl std::fmt::Debug for PositionParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PositionParams")
            .field("board", &self.board.to_fen())
            .finish()
    }
}

pub fn handle_position(bot: &mut Bot, params: PositionParams) -> Option<String> {
    info!("Set position to: {}", params.board.to_fen());
    bot.set_board(params.board);
    None
}

pub fn parse_position(params: &str) -> Result<UCICommand, CommandParseError> {
    let (literal, rest) = params.split_once(" ").ok_or(CommandParseError::ParseError(
        "Missing \"FEN\" or \"startpos\" literal".into(),
    ))?;
    let board = match literal {
        "startpos" => {
            let split = rest.split_once("moves");

            match split {
                Some((_, move_list)) => {
                    let mut board = Board::default();
                    for mv_str in move_list.trim().split_whitespace() {
                        let mov = Move::from_uci_notation(mv_str, &board).map_err(|_| {
                            CommandParseError::ParseError(format!(
                                "Move {} is not valid uci notation",
                                mv_str
                            ))
                        })?;

                        board.make_move(&mov, false, true).map_err(|_| {
                            CommandParseError::ParseError(format!(
                                "Move {:?} is invalid for the position {}",
                                mov,
                                board.to_fen()
                            ))
                        })?;
                    }

                    board
                }
                None => Board::default(),
            }
        }
        "fen" => {
            let split = rest.split_once("moves");

            match split {
                Some((fen, move_list)) => {
                    let mut board = Board::from_fen(fen)
                        .map_err(|err| CommandParseError::ParseError(err.to_string()))?;

                    let mut moves = vec![];
                    for mv_str in move_list.trim().split_whitespace() {
                        let mov = Move::from_uci_notation(mv_str, &board).map_err(|_| {
                            CommandParseError::ParseError(format!(
                                "Move {} is not valid uci notation",
                                mv_str
                            ))
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

                    board
                }
                //rest is the fen
                None => Board::from_fen(rest)
                    .map_err(|err| CommandParseError::ParseError(err.to_string()))?,
            }
        }
        _ => Err(CommandParseError::ParseError(
            format!("Invalid literal : {}", literal).into(),
        ))?,
    };

    Ok(UCICommand::Position(PositionParams { board }))
}
