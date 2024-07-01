use crate::game::{ east, north, north_east, north_west, south, south_east, south_west, west };

const fn calculate_knight_attack_pattern() -> [u64; 64] {
    let mut res = [0; 64];
    let mut i = 0;
    while i < 64 {
        let pos = 1 << i;

        let mut pattern = 0;

        pattern |= east(north(pos, 2), 1);
        pattern |= west(north(pos, 2), 1);
        pattern |= east(south(pos, 2), 1);
        pattern |= west(south(pos, 2), 1);
        pattern |= east(south_east(pos, 1), 1);
        pattern |= east(north_east(pos, 1), 1);
        pattern |= west(south_west(pos, 1), 1);
        pattern |= west(north_west(pos, 1), 1);

        res[i] = pattern;
        i += 1;
    }
    res
}

pub const ATTACK_PATTERN_KNIGHT: [u64; 64] = calculate_knight_attack_pattern();
