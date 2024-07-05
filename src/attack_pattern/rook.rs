use crate::{ game::{ bit_manipulation, Square }, lookup_sliding_piece };

/// Calculates the rook attacks for a given square the rook is on in vertical direction.
pub fn rook_attacks_vertical(enemy_occupied: u64, ally_occupied: u64, sq: Square) -> u64 {
    let mut occupied = enemy_occupied | ally_occupied;
    occupied = bit_manipulation::to_a_file(occupied, sq);
    occupied = bit_manipulation::a_file_to_1_rank(occupied);

    let mut attacks = lookup_sliding_piece!(occupied, 7 - sq.rank());

    attacks = bit_manipulation::rev_a_file_to_1_rank(attacks);
    attacks = bit_manipulation::from_a_file(attacks, sq);

    // Mask out the same color pieces
    attacks ^= attacks & ally_occupied;
    attacks
}

/// Calculates the rook attacks for a given square the rook is on in vertical direction.
pub fn rook_attacks_horizontal(enemy_occupied: u64, ally_occupied: u64, sq: Square) -> u64 {
    let mut occupied = enemy_occupied | ally_occupied;
    occupied = bit_manipulation::to_1_rank(occupied, sq);

    let mut attacks = lookup_sliding_piece!(occupied, sq.file());

    attacks = bit_manipulation::from_1_rank(attacks, sq);

    // Mask out the same color pieces
    attacks ^= attacks & ally_occupied;
    attacks
}
