#![allow(dead_code)]

use chess_bot::{init_logging, uci::start_uci_protocol};

fn main() {
    init_logging();
    start_uci_protocol();
}
