use std::str::FromStr;
use itertools::Itertools;
use thiserror::Error;

use crate::game::Move;

#[derive(Debug)]
pub enum Command {
    Quit,
    UCI,
    IsReady,
    Go,
    Position(Option<String>, Vec<Move>),
    SetOption(String, String),
    Display,
    UCINewGame,
}

#[derive(Error, Debug)]
pub enum CommandParseError {
    #[error("Command not recognized: {0}")] InvalidCommand(String),
    #[error("Missing Parameter: {0}")] MissingParameter(String),
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
            str if str.starts_with("position") => {
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
                    let moves = parts
                        .map(|s| Move::from_source_dest(&s.into()))
                        .collect_vec();
                    Ok(Command::Position(fen, moves))
                } else {
                    Ok(Command::Position(fen, Vec::new()))
                }
            }
            s => Err(CommandParseError::InvalidCommand(s.to_string())),
        }
    }
}

impl Command {
    pub fn is_quit(&self) -> bool {
        match self {
            Command::Quit => true,
            _ => false,
        }
    }

    pub fn is_start(&self) -> bool {
        match self {
            Command::UCI => true,
            _ => false,
        }
    }
}