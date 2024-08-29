#![allow(dead_code)]

use chess_bot::{init, uci::run_uci_protocol};

fn main() {
    init();
    run_uci_protocol();
}
