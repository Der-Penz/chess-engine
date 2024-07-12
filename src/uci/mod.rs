use std::io;

use commands::{ Command, CommandParseError };
use handling::handle_uci_command;
use log::{ info, warn };

use crate::game::Board;

mod commands;
mod handling;

#[derive(Debug, PartialEq)]
pub enum UCISTATE {
    Idle,
    Playing,
}

pub fn start_uci_protocol() {
    let mut board: Board = Board::base();
    let mut state = UCISTATE::Idle;
    loop {
        let command = read_uci_input();
        if let Err(e) = command {
            warn!("Invalid command: {:}", e);
            continue;
        }
        let command = command.unwrap();
        info!("Received Command: {:?}", command);

        if command.is_quit() {
            info!("Quitting UCI Protocol");
            break;
        }
        if command.is_start() && state == UCISTATE::Idle {
            info!("Starting UCI Protocol");
            state = UCISTATE::Playing;
        }

        if state == UCISTATE::Idle {
            info!("Ignoring command {:?} while in idle state", command);
            continue;
        }

        match handle_uci_command(command, &mut board) {
            Some(message) => send_message(&message),
            None => (),
        };
    }
}

/// Reads standard input and parses it into a UCI command
pub fn read_uci_input() -> Result<Command, CommandParseError> {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    info!("Received Message: {:?}", input);
    input.parse::<Command>()
}

/// Sends a message to the output stream
pub fn send_message(message: &str) {
    info!("UCI Send: {:}", message);
    println!("{:}", message);
}
