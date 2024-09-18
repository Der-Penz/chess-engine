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
    pub time_control: Option<TimeControl>,
}

impl GoParams {
    pub fn new(mode: GoMode, time_control: Option<TimeControl>) -> Self {
        GoParams { mode, time_control }
    }
}

#[derive(Debug, PartialEq)]
pub enum GoMode {
    Search(Limits),
    Perft(u8),
    Eval(bool),
}

#[derive(Debug, PartialEq)]
pub struct TimeControl {
    pub w_time: u128,
    pub b_time: u128,
    pub w_inc: u128,
    pub b_inc: u128,
    pub moves_to_go: u8,
}

impl std::default::Default for TimeControl {
    fn default() -> Self {
        TimeControl {
            w_time: 0,
            b_time: 0,
            w_inc: 0,
            b_inc: 0,
            moves_to_go: 0,
        }
    }
}

pub fn handle_go(bot: &mut Bot, params: GoParams) -> Option<String> {
    info!("ET: go with mode {:?}", params.mode);

    let msg = match params.mode {
        GoMode::Search(limit) => {
            bot.think(limit, params.time_control);
            None
        }
        GoMode::Perft(depth) => Some(bot.perft(depth)),
        GoMode::Eval(divide) => Some(bot.eval_board(divide)),
    };

    msg
}

pub fn parse_go(params: &str) -> Result<UCICommand, CommandParseError> {
    let mut limits = Limits::default();
    let mut time_control = TimeControl::default();

    let mut parts = params.split_whitespace();
    while let Some(part) = parts.next() {
        if part == "eval" {
            let divide = parts.next().is_some_and(|val| val == "divide");
            let params = GoParams::new(GoMode::Eval(divide), None);
            return Ok(UCICommand::Go(params));
        }

        if part == "perft" {
            let depth: u8 = parts
                .next()
                .ok_or(CommandParseError::ParseError("Missing depth param".into()))?
                .parse()
                .map_err(|_| CommandParseError::ParseError("Invalid depth".into()))?;
            let params = GoParams::new(GoMode::Perft(depth), None);
            return Ok(UCICommand::Go(params));
        }

        if part == "infinite" || part.is_empty() {
            limits.add_limit(Limit::Depth(INFINITY_DEPTH));
            let params = GoParams::new(GoMode::Search(limits), None);
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
            "wtime" => {
                let w_time = parts
                    .next()
                    .ok_or(CommandParseError::ParseError("Missing wtime param".into()))?
                    .parse()
                    .map_err(|_| CommandParseError::ParseError("Invalid wtime".into()))?;
                time_control.w_time = w_time;
            }
            "btime" => {
                let b_time = parts
                    .next()
                    .ok_or(CommandParseError::ParseError("Missing btime param".into()))?
                    .parse()
                    .map_err(|_| CommandParseError::ParseError("Invalid btime".into()))?;
                time_control.b_time = b_time;
            }
            "winc" => {
                let w_inc = parts
                    .next()
                    .ok_or(CommandParseError::ParseError("Missing winc param".into()))?
                    .parse()
                    .map_err(|_| CommandParseError::ParseError("Invalid winc".into()))?;
                time_control.w_inc = w_inc;
            }
            "binc" => {
                let b_inc = parts
                    .next()
                    .ok_or(CommandParseError::ParseError("Missing binc param".into()))?
                    .parse()
                    .map_err(|_| CommandParseError::ParseError("Invalid binc".into()))?;
                time_control.b_inc = b_inc;
            }
            "searchmoves" => {
                warn!("searchmoves is not implemented yet");
            }
            _ => {
                warn!(
                    "Received invalid search limit \"{}\", skip this limit",
                    part
                );
            }
        };
    }

    let params = GoParams::new(GoMode::Search(limits), Some(time_control));
    Ok(UCICommand::Go(params))
}
