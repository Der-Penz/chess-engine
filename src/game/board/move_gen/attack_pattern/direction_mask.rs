pub const DIRECTION_MASKS: [[u64; 64]; 8] = generate_direction_masks();

/// The offsets for each direction N S W E NW SE NE SW
const DIRECTION_OFFSET: [(isize, isize); 8] = [
    (0, 1),
    (0, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (1, -1),
    (1, 1),
    (-1, -1),
];

const fn generate_direction_masks() -> [[u64; 64]; 8] {
    let mut masks = [[0; 64]; 8];

    let mut dir_idx: usize = 0;
    while dir_idx < 8 {
        let mut square_index: usize = 0;
        while square_index < 64 {
            let (file, rank) = (square_index % 8, square_index / 8);
            let mut i = 0;
            while i < 8 {
                let file = file as isize + DIRECTION_OFFSET[dir_idx].0 * i;
                let rank = rank as isize + DIRECTION_OFFSET[dir_idx].1 * i;
                let coord = rank * 8 + file;
                if coord < 64 && coord >= 0 {
                    masks[dir_idx][square_index] |= 1 << coord;
                } else {
                    break;
                }
                i += 1;
            }
            square_index += 1;
        }

        dir_idx += 1;
    }
    masks
}
