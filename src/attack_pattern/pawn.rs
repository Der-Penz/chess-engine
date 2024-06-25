const fn calculate_pawn_attack_pattern(white: bool) -> [u64; 64] {
    let mut res = [0; 64];
    let mut i = 0;
    while i < 64 {
        let pos = 1 << i;
        if white{
            res[i] = pos << 8;
            if i / 8 == 1{
                res[i] |= pos << 16;
            }
        }else{
            res[i] = pos >> 8;
            if i / 8 == 6{
                res[i] |= pos >> 16;
            }
        }
        i += 1;
    }
    res
}

pub const ATTACK_PATTERN_PAWN_WHITE: [u64; 64] = calculate_pawn_attack_pattern(true);
const ATTACK_PATTERN_PAWN_BLACK: [u64; 64] = calculate_pawn_attack_pattern(false);
pub const ATTACK_PATTERN_PAWN: [[u64; 64]; 2] = [ATTACK_PATTERN_PAWN_WHITE, ATTACK_PATTERN_PAWN_BLACK];