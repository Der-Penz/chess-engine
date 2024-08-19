use std::io;

use commands::{handle_uci_command, CommandParseError, UCICommand};
use log::{info, warn};

use crate::{
    bot::{search::min_max::MinMaxSearch, Bot},
    game::Board,
};

mod commands;

#[derive(Debug, PartialEq)]
pub enum UCIState {
    Idle,
    Playing,
}

pub fn start_uci_protocol() {
    let mut bot = Bot::new(Box::new(MinMaxSearch {}));
    let mut state = UCIState::Idle;
    loop {
        let command = read_uci_input();
        if let Err(e) = command {
            warn!("{}", e);
            continue;
        }
        let command = command.unwrap();
        info!("Received Command: {:?}", command);

        if command == UCICommand::Quit {
            info!("Quitting UCI Protocol");
            break;
        }
        if command == UCICommand::UCI && state == UCIState::Idle {
            info!("Starting UCI Protocol");
            state = UCIState::Playing;
        }

        if state == UCIState::Idle {
            info!("Ignoring command {:?} while in idle state", command);
            continue;
        }

        if let Some(message) = handle_uci_command(command, &mut bot) {
            send_message(&message);
        };
    }
}

/// Reads standard input and parses it into a UCI command
pub fn read_uci_input() -> Result<UCICommand, CommandParseError> {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    info!("Received Message: {:?}", input);
    input.parse::<UCICommand>()
}

/// Sends a message to the output stream
pub fn send_message(message: &str) {
    info!("UCI Send: {:}", message);
    println!("{:}", message);
}
