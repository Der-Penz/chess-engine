const fn generate_sliding_piece_lookup_table() -> [[u8; 8]; 64] {
    let mut lookup_table = [[0; 8]; 64];

    let mut i: usize = 0;
    let mut j: usize = 0;
    while i < 64 {
        //occupied squares
        let o = i << 1;
        while j < 8 {
            //sliding piece square
            let s = 1 << j;

            let positive_ray_attack = o ^ o.overflowing_sub(2 * s).0;

            let s = s.reverse_bits();
            let o = o.reverse_bits();

            let negative_ray_attack = o ^ o.overflowing_sub((2usize).overflowing_mul(s).0).0;
            let negative_ray_attack = negative_ray_attack.reverse_bits();

            let attacks = positive_ray_attack | negative_ray_attack;
            lookup_table[i as usize][j as usize] = (attacks & 0xff) as u8;

            j += 1;
        }
        j = 0;
        i += 1;
    }
    lookup_table
}

///
/// Lookup table for sliding piece attacks
///* First index is the occupied squares in a file
///* Second index is the file of the sliding piece  
/// Returns a file mask of the squares that are attacked by the sliding piece.
/// **For pieces of the same color, the mask is inclusive of the sliding piece's square,
/// so the same color pieces must be masked out.**
pub const SLIDING_ATTACK_LOOKUP_TABLE: [[u8; 8]; 64] = generate_sliding_piece_lookup_table();
