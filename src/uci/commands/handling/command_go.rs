use crate::{
    bot::{
        search::limit::{Limit, Limits},
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
    let (mode, rest) = params.split_once(' ').unwrap_or((params, ""));

    let mode = mode.to_lowercase();
    let rest = rest.to_lowercase();

    if mode == "eval" {
        let divide = rest.trim() == "divide";
        let params = GoParams::new(GoMode::Eval(divide));
        return Ok(UCICommand::Go(params));
    }

    if mode == "perft" {
        let depth = rest
            .parse()
            .map_err(|_| CommandParseError::ParseError("Invalid depth".into()))?;
        let params = GoParams::new(GoMode::Perft(depth));
        return Ok(UCICommand::Go(params));
    }

    let mut limits = Limits::new();
    match &mode[..] {
        //infinite search can only be stopped by the "stop" command
        "infinite" | "" => {
            limits = Limits::new();
            let params = GoParams::new(GoMode::Search(limits));
            return Ok(UCICommand::Go(params));
        }
        "depth" => {
            let depth = rest
                .parse()
                .map_err(|_| CommandParseError::ParseError("Invalid depth".into()))?;

            limits.add_limit(Limit::Depth(depth));
        }
        "nodes" => {
            let nodes = rest
                .parse()
                .map_err(|_| CommandParseError::ParseError("Invalid node count".into()))?;
            limits.add_limit(Limit::NodeCount(nodes));
        }
        "movetime" => {
            let movetime = rest
                .parse()
                .map_err(|_| CommandParseError::ParseError("Invalid movetime".into()))?;
            limits.add_limit(Limit::Time(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis(),
                movetime,
            ));
        }
        _ => {
            return Err(CommandParseError::ParseError(format!(
                "invalid search limit \"{}\"",
                mode
            )))
        }
    };

    let params = GoParams::new(GoMode::Search(limits));
    Ok(UCICommand::Go(params))
}
