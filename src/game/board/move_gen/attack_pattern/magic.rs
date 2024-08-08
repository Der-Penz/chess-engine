pub fn create_movement_mask(square: Square, orthogonal: bool) -> u64 {
    let mut mask = 0u64;

    let tr = square.rank();
    let tf = square.file();

    if orthogonal {
        let mut r = tr + 1;
        let mut f = tf + 1;
        while r <= 6 && f <= 6 {
            mask |= 1 << (r * 8 + f);
            r += 1;
            f += 1;
        }
        let mut r = tr + 1;
        let mut f = tf - 1;
        while r <= 6 && f >= 1 {
            mask |= 1 << (r * 8 + f);
            r += 1;
            f -= 1;
        }
        let mut r = tr - 1;
        let mut f = tf + 1;
        while r >= 1 && f <= 6 {
            mask |= 1 << (r * 8 + f);
            r -= 1;
            f += 1;
        }
        let mut r = tr - 1;
        let mut f = tf - 1;
        while r >= 1 && f >= 1 {
            mask |= 1 << (r * 8 + f);
            r -= 1;
            f -= 1;
        }
    } else {
        for r in (tr + 1)..=6 {
            mask |= 1 << (r * 8 + tf);
        }
        for r in (1..tr).rev() {
            mask |= 1 << (r * 8 + tf);
        }
        for f in (tf + 1)..=6 {
            mask |= 1 << (tr * 8 + f);
        }
        for f in (1..tf).rev() {
            mask |= 1 << (tr * 8 + f);
        }
    }

    mask
}
