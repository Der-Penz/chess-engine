#![allow(dead_code)]

use std::io::Write;

use attack_pattern::SLIDING_ATTACK_LOOKUP_TABLE;
use log::LevelFilter;
#[cfg(feature = "uci")]
use uci::start_uci_protocol;

#[macro_use]
extern crate num_derive;

mod attack_pattern;
mod game;
mod uci;

fn main() {
    #[cfg(not(feature = "log_to_file"))]
    {
        env_logger::builder()
            .format(|buf, record| writeln!(buf, "{}", record.args()))
            .filter(None, LevelFilter::Info)
            .init();
    }

    #[cfg(feature = "log_to_file")]
    {
        let log_file = std::fs::File::create("log.log").unwrap();
        env_logger::Builder::from_default_env()
            .format(|buf, record| writeln!(buf, "[{}] - {}", record.level(), record.args()))
            .target(env_logger::Target::Pipe(Box::new(log_file)))
            .filter(None, LevelFilter::Info)
            .init();
    }

    #[cfg(feature = "uci")]
    start_uci_protocol();
}
