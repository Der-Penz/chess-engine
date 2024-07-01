use crate::game::{ north, north_east, north_west, south, south_east, south_west };

const fn calculate_pawn_attack_pattern(white: bool) -> [u64; 64] {
    let mut res = [0; 64];
    let mut i = 0;
    while i < 64 {
        let pos = 1 << i;
        if white {
            res[i] = north_west(pos, 1) | north_east(pos, 1);
        } else {
            res[i] = south_west(pos, 1) | south_east(pos, 1);
        }
        i += 1;
    }
    res
}

const fn calculate_pawn_move_pattern(white: bool) -> [u64; 64] {
    let mut res = [0; 64];
    let mut i = 0;
    while i < 64 {
        let pos = 1 << i;
        if white {
            res[i] = north(pos, 1);
        } else {
            res[i] = south(pos, 1);
        }
        i += 1;
    }
    res
}

const MOVE_PATTERN_PAWN_WHITE: [u64; 64] = calculate_pawn_move_pattern(true);
const MOVE_PATTERN_PAWN_BLACK: [u64; 64] = calculate_pawn_move_pattern(false);
pub const MOVE_PATTERN_PAWN: [[u64; 64]; 2] = [MOVE_PATTERN_PAWN_WHITE, MOVE_PATTERN_PAWN_BLACK];

const ATTACK_PATTERN_PAWN_WHITE: [u64; 64] = calculate_pawn_attack_pattern(true);
const ATTACK_PATTERN_PAWN_BLACK: [u64; 64] = calculate_pawn_attack_pattern(false);
pub const ATTACK_PATTERN_PAWN: [[u64; 64]; 2] = [ATTACK_PATTERN_PAWN_WHITE, ATTACK_PATTERN_PAWN_BLACK];

