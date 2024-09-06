pub mod command_go;
pub mod command_position;
pub mod command_set_option;
pub mod command_uci;

use command_set_option::OptionType;

use crate::bot::{ActionMessage, Bot};

use super::UCICommand;

pub fn handle_uci_command(command: UCICommand, bot: &mut Bot) -> Option<String> {
    info!("Handling command: {:?}", command);

    match command {
        UCICommand::UCINewGame => {
            bot.send_message(ActionMessage::SetOption(OptionType::ClearHash));
            None
        }
        UCICommand::Quit => None,
        UCICommand::UCI => Some(command_uci::handle_setup().to_string()),
        UCICommand::IsReady => Some("readyok".to_string()),
        UCICommand::SetOption(params) => command_set_option::handle_set_option(bot, params),
        UCICommand::Position(params) => command_position::handle_position(bot, params),
        UCICommand::Display => Some(bot.get_board().to_string()),
        UCICommand::Go(params) => command_go::handle_go(bot, params),
        UCICommand::Stop => {
            bot.stop();
            None
        }
    }
}
