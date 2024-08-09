use crate::game::{board::bit_board::BitBoard, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slider {
    deltas: [(i8, i8); 4],
}

impl Slider {
    pub const ROOK: Slider = Slider {
        deltas: [(1, 0), (0, -1), (-1, 0), (0, 1)],
    };
    pub const BISHOP: Slider = Slider {
        deltas: [(1, 1), (1, -1), (-1, -1), (-1, 1)],
    };

    pub fn moves(&self, square: Square, blockers: BitBoard) -> BitBoard {
        let mut moves = 0;
        for &(df, dr) in &self.deltas {
            let mut ray = square;
            while !blockers.is_occupied(ray) {
                if let Some(shifted) = ray.try_offset(df, dr) {
                    ray = shifted;
                    moves |= ray.to_mask();
                } else {
                    break;
                }
            }
        }
        moves.into()
    }

    pub fn movement_mask(&self, square: Square) -> u64 {
        let mut mask = 0;
        for &(df, dr) in &self.deltas {
            let mut ray = square;
            while let Some(shifted) = ray.try_offset(df, dr) {
                mask |= ray.to_mask();
                ray = shifted;
            }
        }
        mask &= !square.to_mask();
        mask
    }
}
