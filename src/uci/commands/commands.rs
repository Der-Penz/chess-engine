use crate::game::Move;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Quit,
    UCI,
    IsReady,
    Go,
    Position(Option<String>, Vec<Move>),
    SetOption(String, String),
    Display,
    UCINewGame,
    Stop,
}
