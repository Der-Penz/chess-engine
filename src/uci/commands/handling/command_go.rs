use crate::{
    bot::Bot,
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
    Eval(bool),
}

const MAX_DEPTH: u8 = 64;
pub fn handle_go(bot: &mut Bot, params: GoParams) -> Option<String> {
    info!("ET: go with mode {:?}", params.mode);

    let msg = match params.mode {
        GoMode::Depth(depth) => {
            bot.think(depth);
            None
        }
        GoMode::Infinite => {
            bot.think(MAX_DEPTH);
            None
        }
        GoMode::Nodes(_) => todo!("implement nodes mode"),
        GoMode::Perft(depth) => Some(bot.perft(depth)),
        GoMode::Eval(divide) => Some(bot.eval_board(divide)),
    };

    msg
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
        "eval" => {
            let divide = rest == "divide";
            let params = GoParams::new(GoMode::Eval(divide));
            Ok(UCICommand::Go(params))
        }
        _ => Err(CommandParseError::ParseError(format!(
            "mode \"{}\" not implemented",
            mode
        ))),
    }
}
