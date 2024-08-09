use lazy_static::lazy_static;
use precomputed_magic::{BISHOP_MAGICS, BISHOP_TABLE_SIZE, ROOK_MAGICS, ROOK_TABLE_SIZE};
use slider::Slider;

use crate::game::{board::bit_board::BitBoard, Square};

pub mod find_magic;
mod precomputed_magic;
mod rng;
mod slider;

type LookUpTable = [Vec<BitBoard>; 64];

lazy_static! {
    static ref BISHOP_ATTACKS: LookUpTable = init_lookup_table(&Slider::BISHOP);
    static ref ROOK_ATTACKS: LookUpTable = init_lookup_table(&Slider::ROOK);
}

#[derive(Debug)]
struct MagicEntry {
    pub mask: u64,
    pub magic: u64,
    pub shift: u8,
    pub offset: usize,
}

fn magic_index(entry: &MagicEntry, blockers: u64) -> usize {
    let blockers = blockers & entry.mask;
    let hash = blockers.wrapping_mul(entry.magic);
    let index = (hash >> entry.shift) as usize;
    entry.offset + index as usize
}

pub fn get_rook_moves(square: Square, blockers: u64) -> BitBoard {
    ROOK_ATTACKS[square][magic_index(&ROOK_MAGICS[square], blockers)]
}

pub fn get_bishop_moves(square: Square, blockers: u64) -> BitBoard {
    BISHOP_ATTACKS[square][magic_index(&BISHOP_MAGICS[square], blockers)]
}

fn init_lookup_table(slider: &Slider) -> LookUpTable {
    const EMPTY: Vec<BitBoard> = Vec::new();
    let mut attack_table: LookUpTable = [EMPTY; 64];

    let size = if *slider == Slider::ROOK {
        println!("rook");
        ROOK_TABLE_SIZE
    } else {
        println!("BISHOP");
        BISHOP_TABLE_SIZE
    };
    println!("init");
    for square in Square::iter_ah_18() {
        attack_table[square].resize(size, BitBoard::default());

        let magic_entry = if *slider == Slider::ROOK {
            &ROOK_MAGICS[square]
        } else {
            &BISHOP_MAGICS[square]
        };
        let mut blockers = 0u64;

        //go over every blocker combination and insert the correct attack pattern
        //at the fitting index
        loop {
            let attacks = slider.moves(square, blockers.into());
            let index = magic_index(magic_entry, blockers);
            attack_table[square][index] = attacks;

            blockers = blockers.wrapping_sub(magic_entry.mask) & magic_entry.mask;
            if blockers == 0 {
                break;
            }
        }
    }
    println!("init done");
    attack_table
}
