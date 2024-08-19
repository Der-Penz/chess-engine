use crate::{
    bot::Bot,
    game::{board::move_gen::MoveGeneration, Board},
    perft,
    uci::commands::{CommandParseError, UCICommand},
};
use log::info;

#[derive(Debug, PartialEq)]
pub struct GoParams {
    pub mode: GoMode,
}

impl GoParams {
    pub fn new(mode: GoMode) -> Self {
        GoParams { mode }
    }
}

#[derive(Debug, PartialEq)]
pub enum GoMode {
    Depth(u8),
    Infinite,
    Nodes(u64),
    Perft(u8),
}

pub fn handle_go(bot: &mut Bot, params: GoParams) -> Option<String> {
    match params.mode {
        GoMode::Depth(depth) => bot.run(depth),
        GoMode::Infinite => bot.run(u8::MAX),
        GoMode::Nodes(_) => todo!("implement nodes mode"),
        GoMode::Perft(depth) => bot.run_perft(depth),
    }

    info!("Engine started calculation");
    None
}

pub fn parse_go(params: &str) -> Result<UCICommand, CommandParseError> {
    let (mode, rest) = params.split_once(' ').unwrap_or((params, ""));

    match mode {
        "depth" => {
            let depth = rest
                .parse()
                .map_err(|_| CommandParseError::ParseError("Invalid depth".into()))?;
            let params = GoParams::new(GoMode::Depth(depth));
            Ok(UCICommand::Go(params))
        }
        "infinite" => {
            let params = GoParams::new(GoMode::Infinite);
            Ok(UCICommand::Go(params))
        }
        "perft" => {
            let depth = rest
                .parse()
                .map_err(|_| CommandParseError::ParseError("Invalid depth".into()))?;
            let params = GoParams::new(GoMode::Perft(depth));
            Ok(UCICommand::Go(params))
        }
        _ => Err(CommandParseError::ParseError(format!(
            "mode \"{}\" not implemented",
            mode
        ))),
    }
}
