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

pub fn valid_position(pos: u8) -> bool {
    pos > 0 && pos < 64
}

pub fn to_board_bit(pos: u8) -> u64{
    0b1 << pos
}

pub fn match_piece(pos: u8, bit_board: u64) -> bool{
    (bit_board & to_board_bit(pos)) > 0
}