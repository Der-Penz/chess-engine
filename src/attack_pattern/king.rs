use crate::game::{east, north, north_east, north_west, south, south_east, south_west, west};

const fn calculate_king_attack_pattern() -> [u64; 64] {
    let mut res = [0; 64];
    let mut i = 0;
    while i < 64 {
        let pos = 1 << i;

        let mut pattern = 0;
        pattern |= north(pos, 1);
        pattern |= north_east(pos, 1);
        pattern |= east(pos, 1);
        pattern |= south_east(pos, 1);
        pattern |= south(pos, 1);
        pattern |= south_west(pos, 1);
        pattern |= west(pos, 1);
        pattern |= north_west(pos, 1);

        res[i] = pattern;
        i += 1;
    }
    res
}

pub const ATTACK_PATTERN_KING: [u64; 64] = calculate_king_attack_pattern();

pub const CASTLE_FREE_SQUARES: [[u64; 2]; 2] =
    [[0x60, 0xe], [0x6000000000000000, 0xe00000000000000]];
