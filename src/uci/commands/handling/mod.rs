mod command_go;
mod command_position;
mod command_uci;

use std::str::FromStr;

use thiserror::Error;

use crate::game::Board;

use super::Command;

pub fn handle_uci_command(command: Command, board: &mut Board) -> Option<String> {
    match command {
        Command::UCINewGame => None,
        Command::Quit => None,
        Command::UCI => Some(command_uci::handle_setup().to_string()),
        Command::IsReady => Some("readyok".to_string()),
        Command::SetOption(_, _) => None,
        Command::Position(pos, moves) => command_position::handle_position(board, &pos, &moves),
        Command::Display => Some(board.to_string()),
        Command::Go => command_go::handle_go(board),
    }
}

#[derive(Error, Debug)]
pub enum CommandParseError {
    #[error("Command not recognized: {0}")]
    InvalidCommand(String),
    #[error("Missing Parameter: {0}")]
    MissingParameter(String),
    #[error("Parsing error: {0}")]
    ParseError(String),
}

impl FromStr for Command {
    type Err = CommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "quit" => Ok(Command::Quit),
            "uci" => Ok(Command::UCI),
            "isready" => Ok(Command::IsReady),
            "d" => Ok(Command::Display),
            "ucinewgame" => Ok(Command::UCINewGame),
            str if str.starts_with("go") => Ok(Command::Go),
            str if str.starts_with("position") => command_position::parse_position(str),
            _ => Err(CommandParseError::InvalidCommand(s.to_string())),
        }
    }
}
