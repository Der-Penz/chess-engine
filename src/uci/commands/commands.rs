use thiserror::Error;

use super::{
    command_go::{self, GoParams},
    command_position::{self, PositionParams},
};

#[derive(Debug, PartialEq)]
pub enum UCICommand {
    Quit,
    UCI,
    IsReady,
    Go(GoParams),
    Position(PositionParams),
    SetOption(String, String),
    Display,
    UCINewGame,
    Eval,
    Stop,
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

const COMMAND_STR_QUIT: &str = "quit";
const COMMAND_STR_UCI: &str = "uci";
const COMMAND_STR_IS_READY: &str = "isready";
const COMMAND_STR_GO: &str = "go";
const COMMAND_STR_POSITION: &str = "position";
const COMMAND_STR_SET_OPTION: &str = "setoption";
const COMMAND_STR_DISPLAY: &str = "d";
const COMMAND_STR_UCI_NEW_GAME: &str = "ucinewgame";
const COMMAND_STR_STOP: &str = "stop";
const COMMAND_STR_EVAL: &str = "eval";

impl std::str::FromStr for UCICommand {
    type Err = CommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (command, params) = s.trim().split_once(' ').unwrap_or((s, ""));
        match command.trim() {
            COMMAND_STR_QUIT => Ok(UCICommand::Quit),
            COMMAND_STR_UCI => Ok(UCICommand::UCI),
            COMMAND_STR_IS_READY => Ok(UCICommand::IsReady),
            COMMAND_STR_DISPLAY => Ok(UCICommand::Display),
            COMMAND_STR_UCI_NEW_GAME => Ok(UCICommand::UCINewGame),
            COMMAND_STR_STOP => Ok(UCICommand::Stop),
            COMMAND_STR_GO => command_go::parse_go(params),
            COMMAND_STR_SET_OPTION => Ok(UCICommand::SetOption("".into(), "".into()).into()),
            COMMAND_STR_POSITION => command_position::parse_position(params),
            COMMAND_STR_EVAL => Ok(UCICommand::Eval),
            _ => Err(CommandParseError::InvalidCommand(command.to_string())),
        }
    }
}
