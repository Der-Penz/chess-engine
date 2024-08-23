pub mod command_go;
pub mod command_position;
pub mod command_uci;

use log::info;

use crate::bot::Bot;

use super::UCICommand;

pub fn handle_uci_command(command: UCICommand, bot: &mut Bot) -> Option<String> {
    info!("Handling command: {:?}", command);

    match command {
        UCICommand::UCINewGame => None,
        UCICommand::Quit => None,
        UCICommand::UCI => Some(command_uci::handle_setup().to_string()),
        UCICommand::IsReady => Some("readyok".to_string()),
        UCICommand::SetOption(_, _) => None,
        UCICommand::Position(params) => command_position::handle_position(bot, params),
        UCICommand::Display => Some(bot.get_board().to_string()),
        UCICommand::Go(params) => command_go::handle_go(bot, params),
        UCICommand::Eval => Some(format!("Eval: {}", bot.eval_board())),
        UCICommand::Stop => {
            bot.stop();
            None
        }
    }
}
