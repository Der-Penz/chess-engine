use crate::game::{bit_manipulation, square::Square};

use super::lookup_sliding_piece;

/// Calculate the bishop attacks for a given square in the main diagonal direction.
pub fn bishop_attacks_main(enemy_occupied: u64, ally_occupied: u64, sq: Square) -> u64 {
    let mut occupied = enemy_occupied | ally_occupied;

    occupied = bit_manipulation::to_main_diagonal(occupied, sq);
    occupied = bit_manipulation::main_diagonal_to_1_rank(occupied);

    let mut attacks = lookup_sliding_piece(occupied, sq.file());

    attacks = bit_manipulation::rev_main_diagonal_to_1_rank(attacks);
    attacks = bit_manipulation::from_main_diagonal(attacks, sq);

    // Mask out the same color pieces
    attacks ^= attacks & ally_occupied;
    attacks
}

/// Calculate the bishop attacks for a given square in the anti diagonal direction.
pub fn bishop_attacks_anti(enemy_occupied: u64, ally_occupied: u64, sq: Square) -> u64 {
    let mut occupied = enemy_occupied | ally_occupied;

    occupied = bit_manipulation::to_anti_diagonal(occupied, sq);
    occupied = bit_manipulation::mirror_horizontal(occupied);
    occupied = bit_manipulation::main_diagonal_to_1_rank(occupied);

    let mut attacks = lookup_sliding_piece(occupied, 7 - sq.file());

    attacks = bit_manipulation::rev_main_diagonal_to_1_rank(attacks);
    attacks = bit_manipulation::mirror_horizontal(attacks);
    attacks = bit_manipulation::from_anti_diagonal(attacks, sq);

    // Mask out the same color pieces
    attacks ^= attacks & ally_occupied;
    attacks
}
