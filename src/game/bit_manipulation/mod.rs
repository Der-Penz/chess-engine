use super::square::Square;

pub mod constants;

#[inline(always)]
/// Returns the least significant bit of the u64
/// Consider using bit_scan_lsb instead as it is faster
pub fn ls_bit_isolation(board: u64) -> u64 {
    board & board.wrapping_neg()
}

#[inline(always)]
/// Resets the least significant bit of the u64
pub fn ls_bit_reset(board: u64) -> u64 {
    board & (board - 1)
}

#[inline(always)]
/// Returns an iterator over all set bits in the u64
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

#[inline(always)]
/// From https://www.chessprogramming.org/BitScan  De Bruijn Multiplication.
/// Faster Bit Scan than using ls_bit_isolation and then counting the trailing zeros
/// or taking the log2 of the number
pub fn bit_scan_lsb(board: u64) -> u8 {
    return DEBRUIJN_INDEX_64[((board & board.wrapping_neg())
        .overflowing_mul(DEBRUIJN_64)
        .0
        >> 58) as usize];
}

const DEBRUIJN_64: u64 = 0x07edd5e59a4e28c2;
const DEBRUIJN_INDEX_64: [u8; 64] = [
    63, 0, 58, 1, 59, 47, 53, 2, 60, 39, 48, 27, 54, 33, 42, 3, 61, 51, 37, 40, 49, 18, 28, 20, 55,
    30, 34, 11, 43, 14, 22, 4, 62, 57, 46, 52, 38, 26, 32, 41, 50, 36, 17, 19, 29, 10, 13, 21, 56,
    45, 25, 31, 35, 16, 9, 12, 44, 24, 15, 8, 23, 7, 6, 5,
];

#[inline(always)]
/// get the least significant bit as a Square and remove it from the bit board
/// Fails if the bit board is empty
pub fn drop_lsb(bb: &mut u64) -> Square {
    let lsb = bit_scan_lsb(*bb);
    *bb = ls_bit_reset(*bb);
    Square::new(lsb)
}

#[inline(always)]
/// Mirrors the bit board horizontally
pub fn mirror_horizontal(bb: u64) -> u64 {
    let mut bb = bb;
    bb = ((bb >> 1) & K1) | ((bb & K1) << 1);
    bb = ((bb >> 2) & K2) | ((bb & K2) << 2);
    bb = ((bb >> 4) & K3) | ((bb & K3) << 4);
    bb
}

const K1: u64 = 0x5555555555555555;
const K2: u64 = 0x3333333333333333;
const K3: u64 = 0x0f0f0f0f0f0f0f0f;

#[inline(always)]
/// Maps the A File to the 1 rank
pub fn a_file_to_1_rank(board: u64) -> u64 {
    (board & constants::A_FILE)
        .overflowing_mul(constants::MAIN_DIAGONAL)
        .0
        >> 56
}

#[inline(always)]
/// Maps the 1 rank to the A File
pub fn rev_a_file_to_1_rank(board: u64) -> u64 {
    (board.overflowing_mul(constants::MAIN_DIAGONAL).0 >> 7) & constants::A_FILE
}

#[inline(always)]
/// Maps the Main diagonal to the 1 rank
pub fn main_diagonal_to_1_rank(board: u64) -> u64 {
    (board & constants::MAIN_DIAGONAL)
        .overflowing_mul(constants::A_FILE)
        .0
        >> 56
}

#[inline(always)]
/// Maps the 1 rank to the Main diagonal
pub fn rev_main_diagonal_to_1_rank(board: u64) -> u64 {
    board.overflowing_mul(constants::A_FILE).0 & constants::MAIN_DIAGONAL
}
#[inline(always)]
pub fn to_a_file(board: u64, sq: Square) -> u64 {
    (board >> sq.file()) & constants::A_FILE
}

#[inline(always)]
pub fn from_a_file(board: u64, sq: Square) -> u64 {
    (board & constants::A_FILE) << sq.file()
}

#[inline(always)]
pub fn to_1_rank(board: u64, sq: Square) -> u64 {
    (board >> (sq.rank() * 8)) & constants::FIRST_RANK
}

#[inline(always)]
pub fn from_1_rank(board: u64, sq: Square) -> u64 {
    (board & constants::FIRST_RANK) << (sq.rank() * 8)
}

#[inline(always)]
pub fn to_main_diagonal(board: u64, sq: Square) -> u64 {
    let amount = (sq.file() as i8) - (sq.rank() as i8);
    if amount > 0 {
        (board << (amount * 8)) & constants::MAIN_DIAGONAL
    } else {
        (board >> (-amount * 8)) & constants::MAIN_DIAGONAL
    }
}

#[inline(always)]
pub fn from_main_diagonal(board: u64, sq: Square) -> u64 {
    let amount = (sq.file() as i8) - (sq.rank() as i8);
    if amount > 0 {
        (board & constants::MAIN_DIAGONAL) >> (amount * 8)
    } else {
        (board & constants::MAIN_DIAGONAL) << (-amount * 8)
    }
}

#[inline(always)]
pub fn to_anti_diagonal(board: u64, sq: Square) -> u64 {
    let amount = 7 - ((sq.file() as i8) + (sq.rank() as i8));
    if amount > 0 {
        (board << (amount * 8)) & constants::ANTI_DIAGONAL
    } else {
        (board >> (-amount * 8)) & constants::ANTI_DIAGONAL
    }
}

#[inline(always)]
pub fn from_anti_diagonal(board: u64, sq: Square) -> u64 {
    let amount = 7 - ((sq.file() as i8) + (sq.rank() as i8));
    if amount > 0 {
        (board & constants::ANTI_DIAGONAL) >> (amount * 8)
    } else {
        (board & constants::ANTI_DIAGONAL) << (-amount * 8)
    }
}

#[inline(always)]
pub const fn north(board: u64, shifts: u8) -> u64 {
    board << (constants::DIR_N * (shifts as i8))
}

#[inline(always)]
pub const fn south(board: u64, shifts: u8) -> u64 {
    board >> (constants::DIR_S * (shifts as i8))
}

#[inline(always)]
pub const fn east(board: u64, shifts: u8) -> u64 {
    (board << (constants::DIR_E * (shifts as i8))) & constants::NOT_A_FILE
}

#[inline(always)]
pub const fn west(board: u64, shifts: u8) -> u64 {
    (board >> (constants::DIR_W * (shifts as i8))) & constants::NOT_H_FILE
}

#[inline(always)]
pub const fn north_east(board: u64, shifts: u8) -> u64 {
    (board << (constants::DIR_NE * (shifts as i8))) & constants::NOT_A_FILE
}

#[inline(always)]
pub const fn north_west(board: u64, shifts: u8) -> u64 {
    (board << (constants::DIR_NW * (shifts as i8))) & constants::NOT_H_FILE
}

#[inline(always)]
pub const fn south_east(board: u64, shifts: u8) -> u64 {
    (board >> (constants::DIR_SE * (shifts as i8))) & constants::NOT_A_FILE
}

#[inline(always)]
pub const fn south_west(board: u64, shifts: u8) -> u64 {
    (board >> (constants::DIR_SW * (shifts as i8))) & constants::NOT_H_FILE
}
