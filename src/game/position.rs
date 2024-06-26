/**
    Returns the string representation of a given board position
  * Example: position 0 -> A1
 */
pub fn to_field_repr(pos: u8) -> String {
    let row = (pos % 8) + 1;
    let col = pos / 8;

    if !valid_position(pos) {
        panic!("Invalid Board Position");
    }

    match col {
        0 => format!("A{}", row),
        1 => format!("B{}", row),
        2 => format!("C{}", row),
        3 => format!("D{}", row),
        4 => format!("E{}", row),
        5 => format!("F{}", row),
        6 => format!("G{}", row),
        7 => format!("H{}", row),
        _ => panic!("Invalid Board Position"),
    }
}

/**
    Validates a given position index to be in the inclusive range of 0 to 63
 */
pub fn valid_position(pos: u8) -> bool {
    pos > 0 && pos < 64
}

/**
    bit shifts a position to the corresponding u64 mask
 */
pub fn to_board_bit(pos: u8) -> u64 {
    0b1 << pos
}

/**
    checks if a given position of a bit board is set
*/
pub fn match_piece(pos: u8, bit_board: u64) -> bool {
    bit_board & to_board_bit(pos) > 0
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
pub fn iter_set_bits(bit_board: u64) -> impl Iterator<Item = u8>{
    (0..64).filter_map(move |pos|{
        if (bit_board & (1 << pos)) != 0 {
            Some(pos)
        }else{
            None
        }
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
