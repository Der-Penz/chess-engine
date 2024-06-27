use num_traits::FromPrimitive;

#[derive(Clone, Copy, PartialEq, Debug, FromPrimitive)]
#[rustfmt::skip]
pub enum Square {
    A1, A2, A3, A4, A5, A6, A7, A8,
    B1, B2, B3, B4, B5, B6, B7, B8,
    C1, C2, C3, C4, C5, C6, C7, C8,
    D1, D2, D3, D4, D5, D6, D7, D8,
    E1, E2, E3, E4, E5, E6, E7, E8,
    F1, F2, F3, F4, F5, F6, F7, F8,
    G1, G2, G3, G4, G5, G6, G7, G8,
    H1, H2, H3, H4, H5, H6, H7, H8,
}

impl From<Square> for u8 {
    fn from(square: Square) -> u8 {
        square as u8
    }
}

impl From<u8> for Square {
    fn from(value: u8) -> Self {
        Square::from_u8(value).expect("Tried parsing invalid board position as a square")
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let row = ((*self as u8) % 8) + 1;
        let col = (*self as u8) / 8;

        let name = match col {
            0 => format!("A{}", row),
            1 => format!("B{}", row),
            2 => format!("C{}", row),
            3 => format!("D{}", row),
            4 => format!("E{}", row),
            5 => format!("F{}", row),
            6 => format!("G{}", row),
            7 => format!("H{}", row),
            _ => panic!("Invalid Board Position"),
        };
        write!(f, "{}", name)
    }
}

impl Square {
    /**
        Validates a given position index to be in the inclusive range of 0 to 63
    */
    pub fn valid(square: u8) -> bool {
        square > 0 && square < 64
    }
    /**
        bit shifts a position to the corresponding u64 mask
    */
    pub fn to_board_bit(pos: u8) -> u64 {
        0b1 << pos
    }
}

/**
    checks if a given position of a bit board is set
*/
pub fn match_piece(pos: u8, bit_board: u64) -> bool {
    bit_board & Square::to_board_bit(pos) > 0
}

/**
   Returns the index of the next set bit in the u64 if any
*/
pub fn next_set_bit(bit_board: u64) -> Option<usize> {
    for i in 0..64 {
        if (bit_board & (1 << i)) != 0 {
            return Some(i);
        }
    }

    None
}

/**
  Returns a Iterator over all indices of set bits in the bit board
 */
pub fn iter_set_bits(bit_board: u64) -> impl Iterator<Item = u8> {
    (0..64).filter_map(move |pos| {
        if (bit_board & (1 << pos)) != 0 { Some(pos) } else { None }
    })
}

/**
    Creates a chess board with x and o as markers.
    Can be used for debugging and displaying moves
 */
pub fn display_position(x_board: u64, o_fields: u64) {
    let mut repr = String::new();

    repr.push_str(" ");
    for x in 'A'..'I' {
        repr.push_str(&format!(" {x}"));
    }
    repr.push_str("\n");

    for y in 0..8 {
        let y = 7 - y;
        repr.push_str(&format!("{}", y + 1));
        for x in 0..8 {
            if match_piece(x + y * 8, x_board) {
                repr.push_str(&format!(" {}", "x"));
            } else if match_piece(x + y * 8, o_fields) {
                repr.push_str(&format!(" {}", "o"));
            } else {
                repr.push_str(&format!(" {}", " "));
            }
        }
        repr.push_str(&format!("  {}\n", y + 1));
    }

    repr.push_str(" ");
    for x in 'A'..'I' {
        repr.push_str(&format!(" {x}"));
    }

    println!("{repr}");
}
