#![allow(dead_code)]

use std::io::Write;
use log::{ info, LevelFilter };
#[macro_use]
extern crate num_derive;

mod game;
mod attack_pattern;

fn main() {
    #[cfg(not(feature = "log_to_file"))]
    {
        env_logger
            ::builder()
            .format(|buf, record| writeln!(buf, "{}", record.args()))
            .filter(None, LevelFilter::Info)
            .init();
    }

    #[cfg(feature = "log_to_file")]
    {
        let log_file = std::fs::File::create("log.log").unwrap();
        env_logger::Builder
            ::from_default_env()
            .format(|buf, record| { writeln!(buf, "[{}] - {}", record.level(), record.args()) })
            .target(env_logger::Target::Pipe(Box::new(log_file)))
            .filter(None, LevelFilter::Info)
            .init();
    }
}

fn test_move_generation(fen: &str, for_piece: Piece) {
    let board = Board::from_fen(fen).expect("Parsable FEN");
    println!("{}", board);

    let mut attacks = 0;

    let mut counter = 0;
    iter_set_bits(board.get_bbs(&for_piece.1)[for_piece.0]).for_each(|s| {
        let moves = board.get_pseudo_legal_moves(s);
        if let Some(moves) = moves {
            moves.iter().for_each(|m| {
                println!("{}", m);
                counter += 1;
                attacks |= Square::to_board_bit(m.dest());
            });
        }
    });
    println!("{} moves", counter);

    display_position_with_bb(attacks, &board);
}
