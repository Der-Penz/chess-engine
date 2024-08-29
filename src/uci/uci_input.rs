use std::{sync::mpsc::Sender, thread};

use log::{info, warn};

use crate::uci::commands::UCICommand;

use super::commands::CommandParseError;

/// Reads standard input and parses it into a UCI command
pub fn read_uci_input() -> Result<UCICommand, CommandParseError> {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    info!("Received Message: {}", input);
    input.parse::<UCICommand>()
}

/// Spawns a thread that reads UCI commands from standard input
pub fn spawn_input_thread(tx: Sender<UCICommand>) {
    thread::spawn(move || loop {
        let command = read_uci_input();
        let Ok(command) = command else {
            warn!("{}", command.err().unwrap());
            continue;
        };
        info!("Received Command: {:?}", command);

        let should_quit = command == UCICommand::Quit;

        tx.send(command).unwrap();

        if should_quit {
            info!("Quitting UCI Protocol Input Thread");
            break;
        }
    });
}
