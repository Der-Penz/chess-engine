use crate::{
    bot::{
        search::limit::{get_current_millis, Limit, Limits},
        Bot, INFINITY_DEPTH,
    },
    uci::commands::{CommandParseError, UCICommand},
};

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
    Search(Limits),
    Perft(u8),
    Eval(bool),
}

pub fn handle_go(bot: &mut Bot, params: GoParams) -> Option<String> {
    info!("ET: go with mode {:?}", params.mode);

    let msg = match params.mode {
        GoMode::Search(limit) => {
            bot.think(limit);
            None
        }
        GoMode::Perft(depth) => Some(bot.perft(depth)),
        GoMode::Eval(divide) => Some(bot.eval_board(divide)),
    };

    msg
}

pub fn parse_go(params: &str) -> Result<UCICommand, CommandParseError> {
    let mut limits = Limits::new();
    let mut parts = params.split_whitespace();
    while let Some(part) = parts.next() {
        if part == "eval" {
            let divide = parts.next().is_some_and(|val| val == "divide");
            let params = GoParams::new(GoMode::Eval(divide));
            return Ok(UCICommand::Go(params));
        }

        if part == "perft" {
            let depth: u8 = parts
                .next()
                .ok_or(CommandParseError::ParseError("Missing depth param".into()))?
                .parse()
                .map_err(|_| CommandParseError::ParseError("Invalid depth".into()))?;
            let params = GoParams::new(GoMode::Perft(depth));
            return Ok(UCICommand::Go(params));
        }

        match part {
            "infinite" | "" => {
                limits.add_limit(Limit::Depth(INFINITY_DEPTH));
            }
            "depth" => {
                let depth = parts
                    .next()
                    .ok_or(CommandParseError::ParseError("Missing depth param".into()))?
                    .parse()
                    .map_err(|_| CommandParseError::ParseError("Invalid depth".into()))?;

                limits.add_limit(Limit::Depth(depth));
            }
            "nodes" => {
                let nodes = parts
                    .next()
                    .ok_or(CommandParseError::ParseError("Missing nodes param".into()))?
                    .parse()
                    .map_err(|_| CommandParseError::ParseError("Invalid node count".into()))?;
                limits.add_limit(Limit::NodeCount(nodes));
            }
            "movetime" => {
                let movetime = parts
                    .next()
                    .ok_or(CommandParseError::ParseError(
                        "Missing movetime param".into(),
                    ))?
                    .parse()
                    .map_err(|_| CommandParseError::ParseError("Invalid movetime".into()))?;
                limits.add_limit(Limit::Time(get_current_millis(), movetime));
            }
            _ => {
                warn!(
                    "Received invalid search limit \"{}\", skip this limit",
                    part
                );
            }
        };
    }

    let params = GoParams::new(GoMode::Search(limits));
    Ok(UCICommand::Go(params))
}
