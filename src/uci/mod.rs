mod commands;
mod uci_input;

use crate::bot::{search::min_max::MinMaxSearch, Bot, ReactionMessage};
use commands::{handle_uci_command, UCICommand};
use log::{error, info};
use std::sync::mpsc::TryRecvError;
use uci_input::spawn_input_thread;

pub fn run_uci_protocol() {
    let mut bot = Bot::new(MinMaxSearch::new());

    let (tx, rx) = std::sync::mpsc::channel();

    spawn_input_thread(tx);

    loop {
        // Receive a command from the input thread
        match rx.try_recv() {
            Ok(command) => {
                if command == UCICommand::Quit {
                    info!("Quitting UCI Protocol");
                    break;
                }

                if let Some(message) = handle_uci_command(command, &mut bot) {
                    send_message(&message);
                };
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                error!("Input channel disconnected, quitting UCI Protocol");
                break;
            }
        }

        // Check if the bot has a reaction to send
        match bot.poll_reaction() {
            Ok(msg) => match msg {
                ReactionMessage::BestMove(m) => {
                    send_message(&format!("bestmove {}", m));
                }
                ReactionMessage::Info(i) => {
                    send_message(&format!("info {}", i));
                }
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                error!("Reaction channel disconnected, quitting UCI Protocol");
                break;
            }
        }
    }
}

/// Sends a message to the output stream
fn send_message(message: &str) {
    info!("UCI Send: {:}", message);
    println!("{:}", message);
}
