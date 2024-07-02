#![allow(dead_code)]

use game::{ board::Board, display_position_with_bb, iter_set_bits, Piece, Square };
#[macro_use]
extern crate num_derive;

mod game;
mod attack_pattern;

fn main() {
    test_move_generation("4N3/2NqqQ2/2N1P3/1N3N2/4NPp1/P1r4P/PPPPP3/8 b - - 0 11", Piece::white_knight());
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
