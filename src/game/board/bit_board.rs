use crate::game::square::Square;

#[derive(Clone, Copy, PartialEq, Eq)]
// BitBoard is a wrapper around a u64 that represents a chess board
// it allows for easy manipulation of the board
pub struct BitBoard(u64);

impl Default for BitBoard {
    fn default() -> Self {
        Self::new(0)
    }
}

impl BitBoard {
    pub fn new(bb: u64) -> BitBoard {
        BitBoard(bb)
    }

    pub fn set(&mut self, square: &Square) {
        self.0 |= square.to_mask();
    }

    pub fn reset(&mut self, square: &Square) {
        self.0 &= !square.to_mask();
    }

    pub fn update(&mut self, square: &Square, set: bool) {
        if set {
            self.set(square);
        } else {
            self.reset(square);
        }
    }

    pub fn num_pieces(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn is_occupied(&self, square: &Square) -> bool {
        self.0 & square.to_mask() != 0
    }

    pub fn get_occupied(&self) -> impl Iterator<Item = Square> {
        crate::game::bit_manipulation::iter_set_bits(self.0).map(Square::new)
    }
}

impl std::fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Square::iter_ah_81().for_each(|square| {
            if self.is_occupied(&square) {
                let _ = write!(f, "1 ");
            } else {
                let _ = write!(f, "0 ");
            }
            if square.file() == 7 && square.rank() != 0 {
                let _ = write!(f, "\n");
            }
        });
        Ok(())
    }
}

impl From<&[Square]> for BitBoard {
    fn from(squares: &[Square]) -> Self {
        squares.iter().fold(BitBoard::default(), |mut bb, s| {
            bb.set(s);
            bb
        })
    }
}

impl From<u64> for BitBoard {
    fn from(bb: u64) -> Self {
        BitBoard(bb)
    }
}

impl From<BitBoard> for u64 {
    fn from(bb: BitBoard) -> Self {
        bb.0
    }
}

impl std::ops::BitOr for BitBoard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 & rhs.0)
    }
}

impl std::ops::BitXor for BitBoard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 ^ rhs.0)
    }
}

impl std::ops::Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        BitBoard(!self.0)
    }
}

impl std::ops::Deref for BitBoard {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for BitBoard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
