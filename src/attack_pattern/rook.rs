const fn calculate_rook_attack_pattern() -> [u64; 64] {
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

        res[i] = pattern ^ pos;
        i += 1;
    }
    res
}

pub const ATTACK_PATTERN_ROOK: [u64; 64] = calculate_rook_attack_pattern();
