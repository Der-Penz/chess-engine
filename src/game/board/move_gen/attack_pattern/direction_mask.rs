pub const DIRECTION_MASKS: [[u64; 64]; 8] = generate_direction_masks();

const DIRECTION_OFFSET: [isize; 8] = [8, -8, -1, 1, 7, -7, 9, -9];

/// The offsets for each direction N S W E NW SE NE SW
const DIRECTION_OFFSET_2D: [(isize, isize); 8] = [
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
                let file = file as isize + DIRECTION_OFFSET_2D[dir_idx].0 * i;
                let rank = rank as isize + DIRECTION_OFFSET_2D[dir_idx].1 * i;
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

/// The masks for aligning two squares in a straight line (rank, file, or diagonal)
/// If it is not possible to align the two squares in a straight line, the mask is garbage
pub const ALIGN_MASKS: [[u64; 64]; 64] = generate_align_masks();

const fn generate_align_masks() -> [[u64; 64]; 64] {
    let mut align_mask = [[0; 64]; 64];

    let mut square_a = 0;
    while square_a < 64 {
        let mut square_b = 0;
        while square_b < 64 {
            let (file_a, rank_a) = (square_a % 8, square_a / 8);
            let (file_b, rank_b) = (square_b % 8, square_b / 8);
            let delta = (file_b - file_a, rank_b - rank_a);
            let dir = (get_sign(delta.0), get_sign(delta.1));

            let mut i = -8;
            while i < 8 {
                let (file, rank) = (file_a + dir.0 * i, rank_a + dir.1 * i);
                if file >= 0 && file < 8 && rank >= 0 && rank < 8 {
                    let coord = (rank * 8 + file) as usize;
                    align_mask[square_a as usize][square_b as usize] |= 1u64 << coord;
                }
                i += 1;
            }

            square_b += 1;
        }
        square_a += 1;
    }

    align_mask
}

const fn get_sign(x: isize) -> isize {
    if x < 0 {
        -1
    } else if x > 0 {
        1
    } else {
        0
    }
}
