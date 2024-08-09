use crate::game::{board::bit_board::BitBoard, Square};

use super::{magic_index, rng::Rng, slider::Slider, MagicEntry};

pub fn find_magics() {
    find_and_print_all_magics(&Slider::ROOK, "ROOK");
    find_and_print_all_magics(&Slider::BISHOP, "BISHOP");
}

fn find_and_print_all_magics(slider: &Slider, slider_name: &str) {
    println!(
        "pub const {}_MAGICS: [MagicEntry; Square::NUM] = [",
        slider_name
    );
    let mut total_table_size = 0;
    for square in Square::iter_ah_18() {
        let (magic_entry, table) = find_magic(slider, square);
        println!(
            "    MagicEntry {{ mask: 0x{:016X}, magic: 0x{:016X}, shift: {}, offset: {} }},",
            magic_entry.mask, magic_entry.magic, magic_entry.shift, total_table_size
        );
        total_table_size += table.len();
    }
    println!("];");
    println!(
        "pub const {}_TABLE_SIZE: usize = {};",
        slider_name, total_table_size
    );
}

fn find_magic(slider: &Slider, square: Square) -> (MagicEntry, Vec<BitBoard>) {
    let mut rng = Rng::default();
    let movement_mask = slider.movement_mask(square);
    let shift = 64 - movement_mask.count_ones() as u8;
    loop {
        // Magics require a low number of active bits, so we AND
        // by two more random values to cut down on the bits set.
        let magic = rng.next_u64() & rng.next_u64() & rng.next_u64();
        let magic_entry = MagicEntry {
            mask: movement_mask,
            magic,
            shift,
            offset: 0,
        };
        if let Ok(table) = try_make_table(slider, square, &magic_entry) {
            return (magic_entry, table);
        }
    }
}

struct TableFillError;

fn try_make_table(
    slider: &Slider,
    square: Square,
    magic_entry: &MagicEntry,
) -> Result<Vec<BitBoard>, TableFillError> {
    let index_bits = 64 - magic_entry.shift;
    let mut table = vec![BitBoard::default(); 1 << index_bits];

    let mut blockers = BitBoard::default();
    loop {
        let moves = slider.moves(square, blockers);
        let table_entry = &mut table[magic_index(magic_entry, *blockers)];
        if **table_entry == 0 {
            // Write to empty slot
            *table_entry = moves;
        } else if *table_entry != moves {
            // Having two different move sets in the same slot is a hash collision
            return Err(TableFillError);
        }

        // Carry-Rippler trick that enumerates all subsets of the mask, getting us all blockers.
        // https://www.chessprogramming.org/Traversing_Subsets_of_a_Set#All_Subsets_of_any_Set
        blockers = BitBoard::new(blockers.wrapping_sub(magic_entry.mask) & magic_entry.mask);
        if *blockers == 0 {
            // Finished enumerating all blocker configurations
            break;
        }
    }
    Ok(table)
}
