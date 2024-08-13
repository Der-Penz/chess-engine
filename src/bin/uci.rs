#![allow(dead_code)]

use chess_bot::{init, uci::start_uci_protocol};

fn main() {
    init();
    start_uci_protocol();
}
