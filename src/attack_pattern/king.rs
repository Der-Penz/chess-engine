const fn calculate_king_attack_pattern() -> [u64; 64] {
    let mut res = [0; 64];
    let mut i = 0;
    while i < 64 {
        let pos = 1 << i;
        let col = i % 8;
        let row = i / 8;

        let mut pattern = 0;
        if col < 7 {
            pattern |= pos << 1;
        }
        if col > 0 {
            pattern |= pos >> 1;
        }
        if row > 0 {
            pattern |= pos >> 8;
            if col < 7 {
                pattern |= pos >> 7;
            }
            if col > 0 {
                pattern |= pos >> 9;
            }
        }
        if row < 7 {
            pattern |= pos << 8;

            if col > 0 {
                pattern |= pos << 7;
            }
            if col < 7 {
                pattern |= pos << 9;
            }
        }

        res[i] = pattern;
        i += 1;
    }
    res
}

pub const ATTACK_PATTERN_KING: [u64; 64] = calculate_king_attack_pattern();
