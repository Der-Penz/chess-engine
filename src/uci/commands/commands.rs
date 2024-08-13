// use crate::game::Move;

use crate::game::move_notation::Move;

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
