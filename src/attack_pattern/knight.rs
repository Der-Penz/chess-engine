const fn calculate_knight_attack_pattern() -> [u64; 64] {
    let mut res = [0; 64];
    let mut i = 0;
    while i < 64 {
        let pos = 1 << i;
        let col = i % 8;

        let mut pattern = 0;
        if col < 6{
            pattern |= pos << 10;
            pattern |= pos >> 6;
        }
        if col > 1{
            pattern |= pos << 6;
            pattern |= pos >> 10;
        }

        if col < 7{
            pattern |= pos << 17;
            pattern |= pos >> 15;
        }
        if col > 0{
            pattern |= pos << 15;
            pattern |= pos >> 17;
        }

        res[i] = pattern;
        i += 1;
    }
    res
}

pub const ATTACK_PATTERN_KNIGHT: [u64; 64] = calculate_knight_attack_pattern();