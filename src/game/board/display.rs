use crate::game::square::Square;

use super::{bit_board::BitBoard, Board};

pub struct BoardDisplay();

const PICE_CHAR_MAP: [[char; 6]; 2] = [
    ['♟', '♞', '♝', '♜', '♛', '♚'],
    ['♙', '♘', '♗', '♖', '♕', '♔'],
];

impl BoardDisplay {
    pub fn as_ascii(board: &Board) -> String {
        let mut ascii_string = String::with_capacity(1000); //try to preallocate the string
        let row_del = "+---+---+---+---+---+---+---+---+\n";
        let col_del = "| ";
        let mut row_count = 8;

        ascii_string.push_str(row_del);
        for square in Square::iter_ah_81() {
            let piece = board.get_sq_piece(square);
            if let Some(piece) = piece {
                ascii_string.push_str(&format!("{}{} ", col_del, piece));
            } else {
                ascii_string.push_str(&format!("{}  ", col_del));
            }
            if square.file() == 7 {
                ascii_string.push_str(&format!("{}{}\n", col_del, row_count));
                ascii_string.push_str(row_del);
                row_count -= 1;
            }
        }

        ascii_string.push_str("  a   b   c   d   e   f   g   h");
        ascii_string.push_str("\n\n");

        ascii_string.push_str(&format!("{:?} to move\n", board.side_to_move));
        ascii_string.push_str(&format!(
            "Castling rights: {:?}\n",
            board.current_state.castling_rights
        ));
        ascii_string.push_str(&format!("FEN: {}\n", board.to_fen()));
        ascii_string.push_str(&format!("Zobrist: {}", board.current_state.zobrist));

        ascii_string
    }

    fn squares_to_string(square_callback: impl Fn(Square) -> Option<String>) -> String {
        let mut repr = String::with_capacity(1000);
        let row_del = "--+---+---+---+---+---+---+---+---+--\n";

        repr.push_str("  | A | B | C | D | E | F | G | H |  \n");

        repr.push_str(row_del);
        Square::iter_ah_81().for_each(|square| {
            if square.file() == 0 {
                repr.push_str(&format!("{} ", square.rank() + 1));
            }

            repr.push_str(&square_callback(square).unwrap_or("|   ".to_string()));

            if square.file() == 7 {
                repr.push_str(&format!("| {}\n", square.rank() + 1));
                repr.push_str(row_del);
            }
        });

        repr.push_str("  | A | B | C | D | E | F | G | H |  \n");

        return repr;
    }

    pub fn as_board(board: &Board) -> String {
        let mut repr = String::with_capacity(1000);
        repr.push_str(&BoardDisplay::squares_to_string(|square| {
            let piece = board.get_sq_piece(square);
            if let Some(piece) = piece {
                let piece_char = PICE_CHAR_MAP[piece.color()][piece.ptype()];
                Some(format!("| {} ", piece_char))
            } else {
                Some("|   ".to_string())
            }
        }));

        repr.push_str(&format!("{:?} to move\n", board.side_to_move));
        repr.push_str(&format!("FEN: {}\n", board.to_fen()));
        repr
    }

    pub fn as_board_with_attacks(board: &Board, attacks: BitBoard) -> String {
        let mut repr = String::with_capacity(1000);

        repr.push_str(&BoardDisplay::squares_to_string(|square| {
            let piece = board.get_sq_piece(square);

            match (piece, attacks.is_occupied(square)) {
                (Some(piece), true) => {
                    let piece_char = PICE_CHAR_MAP[piece.color()][piece.ptype()];
                    Some(format!("| {}{} ", "\u{035C}", piece_char))
                }
                (Some(piece), false) => {
                    let piece_char = PICE_CHAR_MAP[piece.color()][piece.ptype()];
                    Some(format!("| {} ", piece_char))
                }
                (None, true) => Some("| X ".to_string()),
                (None, false) => Some("|   ".to_string()),
            }
        }));

        repr.push_str(&format!("{:?} to move\n", board.side_to_move));
        repr.push_str(&format!("FEN: {}\n", board.to_fen()));
        repr
    }

    pub fn bit_board_to_string(o_bb: BitBoard, x_bb: BitBoard) -> String {
        let mut repr = String::with_capacity(1000);
        repr.push_str(&BoardDisplay::squares_to_string(|square| {
            match (o_bb.is_occupied(square), x_bb.is_occupied(square)) {
                (true, true) => Some("| \u{035C}O ".to_string()),
                (true, false) => Some("| O ".to_string()),
                (false, true) => Some("| X ".to_string()),
                (false, false) => Some("|   ".to_string()),
            }
        }));

        repr
    }
}
