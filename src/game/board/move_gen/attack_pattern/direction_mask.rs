use crate::game::Square;

pub fn get_direction_mask(square: Square, direction: Direction) -> u64 {
    match direction {
        Direction::North => DIRECTION_MASKS[0][square],
        Direction::South => DIRECTION_MASKS[1][square],
        Direction::West => DIRECTION_MASKS[2][square],
        Direction::East => DIRECTION_MASKS[3][square],
        Direction::NorthWest => DIRECTION_MASKS[4][square],
        Direction::SouthEast => DIRECTION_MASKS[5][square],
        Direction::NorthEast => DIRECTION_MASKS[6][square],
        Direction::SouthWest => DIRECTION_MASKS[7][square],
        Direction::NorthToSouth => DIRECTION_MASKS[0][square] | DIRECTION_MASKS[1][square],
        Direction::WestToEast => DIRECTION_MASKS[2][square] | DIRECTION_MASKS[3][square],
        Direction::NorthWestToSouthEast => DIRECTION_MASKS[4][square] | DIRECTION_MASKS[5][square],
        Direction::NorthEastToSouthWest => DIRECTION_MASKS[6][square] | DIRECTION_MASKS[7][square],
    }
}

const DIRECTION_MASKS: [[u64; 64]; 8] = generate_direction_masks();

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

pub enum Direction {
    North,
    South,
    West,
    East,
    NorthWest,
    SouthEast,
    NorthEast,
    SouthWest,
    NorthToSouth,
    WestToEast,
    NorthWestToSouthEast,
    NorthEastToSouthWest,
}

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
                if file >= 0 && file < 8 && rank >= 0 && rank < 8 {
                    let coord = rank * 8 + file;
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

/// The masks for connecting two squares in a straight line (rank, file, or diagonal)
/// If it is not possible to connect the two squares in a straight line, the mask is 0
/// The mask is exclusive of the two squares
/// If the two squares are the same, the mask is 0
pub const CONNECTION_MASK: [[u64; 64]; 64] = generate_connection_masks();

///generate a line between two squares either horizontally, vertically, or diagonally
///if the slope is not 1 -1, 0, or infinity, the mask should be all 0s
const fn generate_connection_masks() -> [[u64; 64]; 64] {
    let mut connection_masks = [[0; 64]; 64];
    let mut square_a = 0;
    while square_a < 64 {
        let mut square_b = 0;
        while square_b < 64 {
            if square_a == square_b {
                connection_masks[square_a as usize][square_b as usize] = 0;
                square_b += 1;
                continue;
            }

            let (smaller, larger) = if square_a < square_b {
                (square_a, square_b)
            } else {
                (square_b, square_a)
            };

            let (file_smaller, rank_smaller) = (smaller % 8, smaller / 8);
            let (file_larger, rank_larger) = (larger % 8, larger / 8);

            let delta: (i32, i32) = (file_larger - file_smaller, rank_larger - rank_smaller);

            //vertical line
            if delta.0 == 0 {
                let mut i = 1;
                while i < 8 {
                    let (file_new, rank_new) = (file_smaller, rank_smaller + i);

                    if rank_new < 8 && rank_new <= rank_larger {
                        let coord = (rank_new * 8 + file_new) as usize;
                        connection_masks[square_a as usize][square_b as usize] |= 1u64 << coord;
                    } else {
                        break;
                    }
                    i += 1;
                }
            }
            // horizontal line
            else if delta.1 == 0 {
                let mut i = 1;
                while i < 8 {
                    let (file_new, rank_new) = (file_smaller + i, rank_smaller);

                    if file_new < 8 && file_new <= file_larger {
                        let coord = (rank_new * 8 + file_new) as usize;
                        connection_masks[square_a as usize][square_b as usize] |= 1u64 << coord;
                    } else {
                        break;
                    }
                    i += 1;
                }
            }
            // diagonal line
            else if delta.0.abs() == delta.1.abs() {
                let mut i = 1;

                let dir = delta.1 / delta.0;

                if dir < 0 {
                    while i < 8 {
                        let (file_new, rank_new) = (file_smaller - i, rank_smaller + i);

                        if file_new >= 0
                            && rank_new < 8
                            && file_new >= file_larger
                            && rank_new <= rank_larger
                        {
                            let coord = (rank_new * 8 + file_new) as usize;
                            connection_masks[square_a as usize][square_b as usize] |= 1u64 << coord;
                        } else {
                            break;
                        }
                        i += 1;
                    }
                } else {
                    while i < 8 {
                        let (file_new, rank_new) = (file_smaller + i, rank_smaller + i);

                        if file_new < 8
                            && rank_new < 8
                            && file_new <= file_larger
                            && rank_new <= rank_larger
                        {
                            let coord = (rank_new * 8 + file_new) as usize;
                            connection_masks[square_a as usize][square_b as usize] |= 1u64 << coord;
                        } else {
                            break;
                        }
                        i += 1;
                    }
                }
            }
            //any other slope where no line can be calculated
            else {
                connection_masks[square_a as usize][square_b as usize] = 0;
            }

            square_b += 1;
        }
        square_a += 1;
    }

    connection_masks
}
