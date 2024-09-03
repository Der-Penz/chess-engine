use crate::{
    bot::{ActionMessage, Bot},
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

    let (name, value) = rest.split_once("value").ok_or_else(|| {
        CommandParseError::ParseError("Missing value literal in setoption command".into())
    })?;

    let option = match name.trim() {
        "Clear Hash" => OptionType::ClearHash,
        _ => {
            return Err(CommandParseError::ParseError(
                format!("Unknown option :{}", name).into(),
            ))
        }
    };

    Ok(UCICommand::SetOption(SetOptionParams { option }))
}
