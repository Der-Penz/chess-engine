const fn calculate_bishop_attack_pattern() -> [u64; 64] {
    let mut res = [0; 64];
    let mut i: usize = 0;
    while i < 64 {
        let col = i % 8;
        let row = i / 8;

        let mut pattern = 0;

        let bottom_left_top_right = 0x8040201008040201u64;
        let top_left_bottom_right = 0x102040810204080u64;

        // let dir = col as isize - row as isize;
        // if dir > 0{
        //     pattern |= bottom_left_top_right >> (dir * 8);
        // }else{
        //     pattern |= bottom_left_top_right << (-dir * 8);
        // }

        let distance = if row + col < 7 - row { (row + col) as u8 } else { (7 - row) as u8 };
        // let distance = (row + col).min(7 - row) as u8;
        if row == col {
            pattern |= top_left_bottom_right;
        } else if row + col < 7 {
            pattern |= top_left_bottom_right << (distance * 8);
        } else {
            pattern |= top_left_bottom_right >> (distance * 8);
        }

        res[i] = pattern;
        i += 1;
    }
    res
}

pub const ATTACK_PATTERN_BISHOP: [u64; 64] = calculate_bishop_attack_pattern();
