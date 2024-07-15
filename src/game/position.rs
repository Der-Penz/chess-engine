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

impl From<&str> for Square {
    fn from(value: &str) -> Self {
        let file: u32 =
            (value.to_lowercase().chars().nth(0).unwrap().to_ascii_lowercase() as u32) - 97;
        let rank: u32 = value.chars().nth(1).unwrap().to_digit(10).unwrap() - 1;
        let square = rank * 8 + file;
        Square::from(square as u8)
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let name = match self.file() {
            0 => format!("a{}", self.rank() + 1),
            1 => format!("b{}", self.rank() + 1),
            2 => format!("c{}", self.rank() + 1),
            3 => format!("d{}", self.rank() + 1),
            4 => format!("e{}", self.rank() + 1),
            5 => format!("f{}", self.rank() + 1),
            6 => format!("g{}", self.rank() + 1),
            7 => format!("h{}", self.rank() + 1),
            _ => panic!("Invalid Board Position"),
        };
        write!(f, "{}", name)
    }
}

impl Square {
    /// Validates a given position index to be in the inclusive range of 0 to 63
    pub fn valid(square: u8) -> bool {
        square < 64
    }

    /// Bit shifts a position to the corresponding u64 mask
    pub fn to_board_bit(square: u8) -> u64 {
        0b1 << square
    }

    /// Returns an iterator over all squares on the board
    /// from A8 to H8, A7 to H7, ..., A1 to H1
    pub fn iter_ah_81() -> impl Iterator<Item = Square> {
        (0..64).map(|square| {
            let square = 63 - (square as u8);
            let col = 7 - (square % 8);
            let row = square / 8;
            let square = col + row * 8;
            Square::from(square)
        })
    }

    /// Returns an iterator over all squares on the board
    /// from A1 to H1, A2 to H2, ..., A8 to H8
    pub fn iter_ah_18() -> impl Iterator<Item = Square> {
        (0..64).map(|square| Square::from(square))
    }

    /// zero indexed row
    pub fn rank(&self) -> u8 {
        (*self as u8) / 8
    }

    /// zero indexed column
    pub fn file(&self) -> u8 {
        (*self as u8) % 8
    }

    /// checks if the given bit board contains the square
    pub fn matches(&self, bb: u64) -> bool {
        bb & Square::to_board_bit((*self).into()) > 0
    }
}

/// checks if a given position of a bit board is set
pub fn match_piece(pos: u8, bit_board: u64) -> bool {
    bit_board & Square::to_board_bit(pos) > 0
}
