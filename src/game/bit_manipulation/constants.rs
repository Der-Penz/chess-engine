pub const EMPTY: u64 = 0;
pub const UNIVERSAL: u64 = !0;

pub const NOT_A_FILE: u64 = 0xfefefefefefefefe;
pub const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f;

pub const A_FILE: u64 = !NOT_A_FILE;
pub const H_FILE: u64 = !NOT_H_FILE;

pub const FIRST_RANK: u64 = 0xff;
pub const NOT_FIRST_RANK: u64 = !FIRST_RANK;

pub const MAIN_DIAGONAL: u64 = 0x8040201008040201;
pub const ANTI_DIAGONAL: u64 = 0x0102040810204080;

pub const DIR_NW: i8 = 7;
pub const DIR_N: i8 = 8;
pub const DIR_NE: i8 = 9;
pub const DIR_E: i8 = 1;
pub const DIR_SE: i8 = 7;
pub const DIR_S: i8 = 8;
pub const DIR_SW: i8 = 9;
pub const DIR_W: i8 = 1;