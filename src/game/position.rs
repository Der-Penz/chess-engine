use num_traits::FromPrimitive;

#[derive(Clone, Copy, PartialEq, Debug, FromPrimitive)]
#[rustfmt::skip]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,    
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
        let col = (*self as u8) % 8;
        let row = ((*self as u8) / 8) + 1;

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
