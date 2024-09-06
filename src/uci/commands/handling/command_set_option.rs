use crate::{
    bot::{
        search::transposition_table::{DEFAULT_HASH_SIZE, MAX_HASH_SIZE},
        ActionMessage, Bot,
    },
    uci::commands::{CommandParseError, UCICommand},
};

#[derive(Debug, PartialEq)]
pub struct SetOptionParams {
    pub option: OptionType,
}

#[derive(Debug, PartialEq)]
pub enum OptionType {
    HashSize(f64),
    ClearHash,
    Threads(u8),
    DebugFile(String),
}

impl OptionType {
    pub fn get_option_description(&self) -> String {
        match self {
            OptionType::HashSize(_) => {
                format!(
                    "Hash type spin default {} min 1 max {}",
                    DEFAULT_HASH_SIZE, MAX_HASH_SIZE
                )
            }
            OptionType::ClearHash => "Clear Hash type button".into(),
            OptionType::Threads(_) => format!("Threads type spin default 1 min 1 max 255"),
            OptionType::DebugFile(_) => format!(
                "Debug Log File type string default {}",
                std::env::var("LOG_FILE").unwrap_or("logs.log".to_string())
            ),
        }
    }

    pub fn get_all_descriptions() -> Vec<String> {
        vec![
            OptionType::HashSize(DEFAULT_HASH_SIZE).get_option_description(),
            OptionType::ClearHash.get_option_description(),
            OptionType::Threads(1).get_option_description(),
            OptionType::DebugFile("".into()).get_option_description(),
        ]
    }
}

pub fn handle_set_option(bot: &mut Bot, params: SetOptionParams) -> Option<String> {
    bot.send_message(ActionMessage::SetOption(params.option));
    None
}

pub fn parse_set_option(params: &str) -> Result<UCICommand, CommandParseError> {
    if !params.starts_with("name") {
        return Err(CommandParseError::ParseError(
            "Missing name literal in setoption command".into(),
        ));
    }

    let rest = params.trim_start_matches("name").trim();

    let (name, value) = rest.split_once("value").unwrap_or_else(|| (rest, ""));

    let value = value.trim();

    let option = match name.trim() {
        "Clear Hash" => OptionType::ClearHash,
        "Hash" => {
            let value = value
                .parse::<f64>()
                .map_err(|_| CommandParseError::ParseError("Invalid value for Hash size".into()))?;
            OptionType::HashSize(value)
        }
        "Threads" => {
            let value = value
                .parse::<u8>()
                .map_err(|_| CommandParseError::ParseError("Invalid value for Threads".into()))?;
            OptionType::Threads(value)
        }
        "Debug Log File" => OptionType::DebugFile(value.into()),
        _ => {
            return Err(CommandParseError::ParseError(
                format!("Unknown option :{}", name).into(),
            ))
        }
    };

    Ok(UCICommand::SetOption(SetOptionParams { option }))
}
