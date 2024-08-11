use itertools::Itertools;
use thiserror::Error;

/// newtype for a square on the chess board
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Square(u8);

impl Square {
    pub const NUM: usize = 64;

    pub fn new(square: u8) -> Square {
        if square < 64 {
            Square(square)
        } else {
            panic!("Invalid square")
        }
    }

    pub fn valid(square: u8) -> bool {
        square < 64
    }

    /// Returns an iterator over all squares on the board
    /// from A8 to H8, A7 to H7, ..., A1 to H1
    pub fn iter_ah_81() -> impl Iterator<Item = Square> {
        (0u8..64u8).map(|square| {
            let square = 63 - (square as u8);
            let col = 7 - (square % 8);
            let row = square / 8;
            let square = col + row * 8;
            Square::new(square)
        })
    }

    /// Returns an iterator over all squares on the board
    /// from A1 to H1, A2 to H2, ..., A8 to H8
    pub fn iter_ah_18() -> impl Iterator<Item = Square> {
        (0u8..64u8).map(|square| Square::new(square))
    }

    /// Bit shifts the square to the corresponding u64 mask
    pub fn to_mask(&self) -> u64 {
        0b1 << self.0
    }

    /// zero indexed
    pub fn rank(&self) -> u8 {
        self.0 / 8
    }

    /// zero indexed
    pub fn file(&self) -> u8 {
        self.0 % 8
    }

    pub fn rank_str(&self) -> String {
        (self.rank() + 1).to_string()
    }

    pub fn file_str(&self) -> String {
        FILES[self.file() as usize].to_string()
    }

    pub fn square_value(&self) -> u8 {
        self.0
    }

    pub fn try_offset(self, file_offset: i8, rank_offset: i8) -> Option<Square> {
        let new_file = self.file() as i8 + file_offset;
        let new_rank = self.rank() as i8 + rank_offset;

        if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
            Some(Square::new((new_file + new_rank * 8) as u8))
        } else {
            None
        }
    }
}

pub const FILES: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];
impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{}", FILES[self.file() as usize], self.rank() + 1)
    }
}

impl std::fmt::Debug for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)?;
        write!(f, " (val:{})", self.0)
    }
}

#[derive(Error, Debug)]
pub enum SquareError {
    #[error("Invalid square")]
    InvalidSquare,
    #[error("Invalid square notation. must be in the format [a-h][1-8] length 2")]
    InvalidSquareNotation,
}

impl TryFrom<&str> for Square {
    type Error = SquareError;

    fn try_from(value: &str) -> Result<Self, self::SquareError> {
        if let Some((file, rank)) = value.chars().collect_tuple() {
            let file = (file.to_ascii_lowercase() as u32) - ('a' as u32);
            let rank = rank.to_digit(10).unwrap() - 1;
            let square = (rank * 8 + file) as u8;
            if Square::valid(square) {
                return Ok(Square::new(square));
            } else {
                return Err(SquareError::InvalidSquare);
            }
        } else {
            return Err(SquareError::InvalidSquareNotation);
        }
    }
}

impl TryFrom<u8> for Square {
    type Error = SquareError;

    fn try_from(value: u8) -> Result<Self, self::SquareError> {
        if Square::valid(value) {
            Ok(Square::new(value))
        } else {
            Err(SquareError::InvalidSquare)
        }
    }
}
impl<T, const N: usize> std::ops::Index<Square> for [T; N] {
    type Output = T;

    fn index(&self, index: Square) -> &Self::Output {
        &self[index.0 as usize]
    }
}

impl<T, const N: usize> std::ops::IndexMut<Square> for [T; N] {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self[index.0 as usize]
    }
}

impl std::ops::Add<u8> for Square {
    type Output = Square;

    fn add(self, rhs: u8) -> Self::Output {
        Square::new(self.0 + rhs)
    }
}

impl std::ops::Sub<u8> for Square {
    type Output = Square;

    fn sub(self, rhs: u8) -> Self::Output {
        Square::new(self.0 - rhs)
    }
}

impl std::ops::Add<i8> for Square {
    type Output = Square;

    fn add(self, rhs: i8) -> Self::Output {
        Square::new((self.0 as i8 + rhs) as u8)
    }
}

impl std::ops::Sub<i8> for Square {
    type Output = Square;

    fn sub(self, rhs: i8) -> Self::Output {
        Square::new((self.0 as i8 - rhs) as u8)
    }
}

//CONSTANTS
impl Square {
    pub const A1: Square = Square(0);
    pub const B1: Square = Square(1);
    pub const C1: Square = Square(2);
    pub const D1: Square = Square(3);
    pub const E1: Square = Square(4);
    pub const F1: Square = Square(5);
    pub const G1: Square = Square(6);
    pub const H1: Square = Square(7);
    pub const A2: Square = Square(8);
    pub const B2: Square = Square(9);
    pub const C2: Square = Square(10);
    pub const D2: Square = Square(11);
    pub const E2: Square = Square(12);
    pub const F2: Square = Square(13);
    pub const G2: Square = Square(14);
    pub const H2: Square = Square(15);
    pub const A3: Square = Square(16);
    pub const B3: Square = Square(17);
    pub const C3: Square = Square(18);
    pub const D3: Square = Square(19);
    pub const E3: Square = Square(20);
    pub const F3: Square = Square(21);
    pub const G3: Square = Square(22);
    pub const H3: Square = Square(23);
    pub const A4: Square = Square(24);
    pub const B4: Square = Square(25);
    pub const C4: Square = Square(26);
    pub const D4: Square = Square(27);
    pub const E4: Square = Square(28);
    pub const F4: Square = Square(29);
    pub const G4: Square = Square(30);
    pub const H4: Square = Square(31);
    pub const A5: Square = Square(32);
    pub const B5: Square = Square(33);
    pub const C5: Square = Square(34);
    pub const D5: Square = Square(35);
    pub const E5: Square = Square(36);
    pub const F5: Square = Square(37);
    pub const G5: Square = Square(38);
    pub const H5: Square = Square(39);
    pub const A6: Square = Square(40);
    pub const B6: Square = Square(41);
    pub const C6: Square = Square(42);
    pub const D6: Square = Square(43);
    pub const E6: Square = Square(44);
    pub const F6: Square = Square(45);
    pub const G6: Square = Square(46);
    pub const H6: Square = Square(47);
    pub const A7: Square = Square(48);
    pub const B7: Square = Square(49);
    pub const C7: Square = Square(50);
    pub const D7: Square = Square(51);
    pub const E7: Square = Square(52);
    pub const F7: Square = Square(53);
    pub const G7: Square = Square(54);
    pub const H7: Square = Square(55);
    pub const A8: Square = Square(56);
    pub const B8: Square = Square(57);
    pub const C8: Square = Square(58);
    pub const D8: Square = Square(59);
    pub const E8: Square = Square(60);
    pub const F8: Square = Square(61);
    pub const G8: Square = Square(62);
    pub const H8: Square = Square(63);
}
