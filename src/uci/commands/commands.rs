use thiserror::Error;

use super::{
    command_go::{self, GoParams},
    command_position::{self, PositionParams},
    command_set_option::{self, SetOptionParams},
};

#[derive(Debug)]
pub enum UCICommand {
    Quit,
    UCI,
    IsReady,
    Go(GoParams),
    Position(PositionParams),
    SetOption(SetOptionParams),
    Display,
    UCINewGame,
    Stop,
}

impl UCICommand {
    pub fn is_quit(&self) -> bool {
        matches!(self, UCICommand::Quit)
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

const COMMAND_STR_QUIT: &str = "quit";
const COMMAND_STR_UCI: &str = "uci";
const COMMAND_STR_IS_READY: &str = "isready";
const COMMAND_STR_GO: &str = "go";
const COMMAND_STR_POSITION: &str = "position";
const COMMAND_STR_SET_OPTION: &str = "setoption";
const COMMAND_STR_DISPLAY: &str = "d";
const COMMAND_STR_UCI_NEW_GAME: &str = "ucinewgame";
const COMMAND_STR_STOP: &str = "stop";

impl std::str::FromStr for UCICommand {
    type Err = CommandParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut com = s;
        loop {
            let (command, params) = com.trim().split_once(' ').unwrap_or((com, ""));

            if command.is_empty() {
                return Err(CommandParseError::InvalidCommand(s.to_string()));
            }

            match command.trim() {
                COMMAND_STR_QUIT => return Ok(UCICommand::Quit),
                COMMAND_STR_UCI => return Ok(UCICommand::UCI),
                COMMAND_STR_IS_READY => return Ok(UCICommand::IsReady),
                COMMAND_STR_DISPLAY => return Ok(UCICommand::Display),
                COMMAND_STR_UCI_NEW_GAME => return Ok(UCICommand::UCINewGame),
                COMMAND_STR_STOP => return Ok(UCICommand::Stop),
                COMMAND_STR_GO => return command_go::parse_go(params),
                COMMAND_STR_SET_OPTION => return command_set_option::parse_set_option(params),
                COMMAND_STR_POSITION => return command_position::parse_position(params),
                _ => (),
            }
            com = params;
        }
    }
}
