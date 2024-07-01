mod constants;

pub use constants::*;

/**
    Returns the least significant bit of the u64
    Consider using bit_scan_lsb instead as it is faster
 */
pub fn ls_bit_isolation(board: u64) -> u64 {
    board & board.wrapping_neg()
}

/**
    Resets the least significant bit of the u64
 */
pub fn ls_bit_reset(board: u64) -> u64 {
    board & (board - 1)
}

/**
    Returns an iterator over all set bits in the u64
 */
pub fn iter_set_bits(mut board: u64) -> impl Iterator<Item = u8> {
    std::iter::from_fn(move || {
        if board == 0 {
            return None;
        }
        let index = bit_scan_lsb(board);
        board = ls_bit_reset(board);
        Some(index)
    })
}

/**
 From https://www.chessprogramming.org/BitScan.
 De Bruijn Multiplication.
 Faster Bit Scan than using ls_bit_isolation and then counting the trailing zeros
 or taking the log2 of the number
 */
pub fn bit_scan_lsb(board: u64) -> u8 {
    return DEBRUIJN_INDEX_64[(((board & board.wrapping_neg()).overflowing_mul(DEBRUIJN_64)).0 >> 58) as usize];
}

const DEBRUIJN_64: u64 = 0x07edd5e59a4e28c2;
const DEBRUIJN_INDEX_64: [u8; 64] = [
    63, 0, 58, 1, 59, 47, 53, 2, 60, 39, 48, 27, 54, 33, 42, 3, 61, 51, 37, 40, 49, 18, 28, 20, 55, 30,
    34, 11, 43, 14, 22, 4, 62, 57, 46, 52, 38, 26, 32, 41, 50, 36, 17, 19, 29, 10, 13, 21, 56, 45,
    25, 31, 35, 16, 9, 12, 44, 24, 15, 8, 23, 7, 6, 5,
];

/**
 Maps the A File to the 1 rank
 */
pub fn a_file_to_1_rank(board: u64) -> u64 {
    (board & A_FILE).overflowing_mul(MAIN_DIAGONAL).0 >> 56
}

/**
 Maps the Main diagonal to the 1 rank
 */
pub fn main_diagonal_to_1_rank(board: u64) -> u64 {
    (board & MAIN_DIAGONAL).overflowing_mul(A_FILE).0 >> 56
}

pub const fn north(board: u64, shifts: u8) -> u64 {
    board << (DIR_N * (shifts as i8))
}

pub const fn south(board: u64, shifts: u8) -> u64 {
    board >> (DIR_S * (shifts as i8))
}

pub const fn east(board: u64, shifts: u8) -> u64 {
    (board << (DIR_E * (shifts as i8))) & NOT_A_FILE
}

pub const fn west(board: u64, shifts: u8) -> u64 {
    (board >> (DIR_W * (shifts as i8))) & NOT_H_FILE
}

pub const fn north_east(board: u64, shifts: u8) -> u64 {
    (board << (DIR_NE * (shifts as i8))) & NOT_A_FILE
}

pub const fn north_west(board: u64, shifts: u8) -> u64 {
    (board << (DIR_NW * (shifts as i8))) & NOT_H_FILE
}

pub const fn south_east(board: u64, shifts: u8) -> u64 {
    (board >> (DIR_SE * (shifts as i8))) & NOT_A_FILE
}

pub const fn south_west(board: u64, shifts: u8) -> u64 {
    (board >> (DIR_SW * (shifts as i8))) & NOT_H_FILE
}
