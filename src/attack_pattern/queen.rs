const fn calculate_queen_attack_pattern() -> [u64; 64] {
    let mut res = [0; 64];
    let mut i = 0;
    while i < 64 {
        let pos = 1 << i;
        let col = i % 8;
        let row = i / 8;

        let mut pattern = 0;

        
        let vertical = 0xff;
        let vertical = vertical << (row * 8);
        pattern |= vertical;

        let horizontal = 0x101010101010101;
        let horizontal = horizontal << (col);
        pattern |= horizontal;

        let bottom_left_top_right = 0x8040201008040201u64;
        let top_left_bottom_right = 0x102040810204080u64;

        let dir = (col as isize) - (row as isize);
        if dir > 0 {
            pattern |= bottom_left_top_right >> (dir * 8);
        } else {
            pattern |= bottom_left_top_right << (-dir * 8);
        }

        let dir = -(col as isize) - (row as isize) + 7;
        if dir > 0 && dir < 7 {
            pattern |= top_left_bottom_right >> (dir * 8);
        }
        if dir <= 0 && dir > -7 {
            pattern |= top_left_bottom_right << (-dir * 8);
        }


        res[i] = pattern ^ pos;
        i += 1;
    }
    res
}

pub const ATTACK_PATTERN_QUEEN: [u64; 64] = calculate_queen_attack_pattern();
