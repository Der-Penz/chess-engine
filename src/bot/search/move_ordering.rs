use crate::game::{board::move_gen::LegalMoveList, Board, Move, PieceType};

use super::{pv_line::PVLine, transposition_table::TranspositionTable};

const MAX_NUMBER_OF_MOVES_PER_POSITION: usize = 218;
type Score = u16;
type MoveIdx = usize;

/// #### Move ordering
/// The move ordering is used to order the moves in the move list to improve the alpha-beta pruning  
/// The move ordering is based on the following heuristics:
/// - PV line: Moves in the PV line are given a high score
/// - Hash move: The move stored in the transposition table is given a high score (most likely overlap with the PV line)
/// - MVV-LVA: Most Valuable Victim - Least Valuable Aggressor table is used to order captures
/// - Pawn promotion: Promotions are given a higher score (depending on the promoted piece)
/// - Move to danger: Moves that put the piece in danger are given a lower score
/// - Move from danger: Moves that move the piece from a danger are given a higher score
/// - Piece value: The value of the piece is added to the score
///  All scores are added
///
/// Moves are lazily picked from the move list based on the score  
/// The move ordering struct is exhausted after all moves have been played and can't be reused
pub struct MoveOrdering<'a> {
    scored: [(Score, MoveIdx); MAX_NUMBER_OF_MOVES_PER_POSITION],
    cur_idx: usize,
    legal_moves: &'a LegalMoveList,
}

impl<'a> MoveOrdering<'a> {
    #[inline(always)]
    fn inc_score(&mut self, idx: usize, inc: u16) {
        self.scored[idx].0 += inc;
    }

    #[inline(always)]
    fn dec_score(&mut self, idx: usize, inc: u16) {
        self.scored[idx].0 = self.scored[idx].0.saturating_sub(inc);
    }

    /// Scores the moves in the move list based on the heuristics mentioned above and creates a move ordering struct
    pub fn score_moves(
        moves: &'a LegalMoveList,
        pv_lines: &Vec<PVLine>,
        ply_from_root: u8,
        board: &Board,
        tt_move: Option<Move>,
    ) -> MoveOrdering<'a> {
        let mut move_ordering = MoveOrdering {
            legal_moves: moves,
            scored: [(0, 0); MAX_NUMBER_OF_MOVES_PER_POSITION],
            cur_idx: 0,
        };

        for (i, mv) in moves.iter().enumerate() {
            //check for the move in the pv line
            if MoveOrdering::in_pv_line(mv, pv_lines, ply_from_root) {
                move_ordering.inc_score(i, 1000);
            }

            //Hash move
            if tt_move == Some(*mv) {
                move_ordering.inc_score(i, 999);
            }

            let moved_piece = board
                .get_sq_piece_variation(mv.source())
                .expect("Move has no piece");

            //capture moves
            if let Some(victim) = board.get_sq_piece_variation(mv.dest()) {
                move_ordering.inc_score(i, MVV_LVA_TABLE[victim][moved_piece]);
            }

            //pawn promotion
            if let Some(promotion_type) = mv.flag().promotion_type() {
                let score = match promotion_type {
                    PieceType::Queen => 105,
                    PieceType::Rook => 104,
                    PieceType::Bishop => 103,
                    PieceType::Knight => 102,
                    _ => 0,
                };
                move_ordering.inc_score(i, score);
            }

            //decrease score if the move puts the piece in danger
            if moves.get_masks().king_danger & mv.dest().to_mask() != 0 {
                move_ordering.dec_score(i, 1);
            }

            //increase score if the move was at a dangerous square
            if moves.get_masks().king_danger & mv.source().to_mask() != 0 {
                move_ordering.inc_score(i, 1);
            }

            //default piece value
            move_ordering.inc_score(i, Into::<usize>::into(moved_piece) as u16);

            //store the index of the move
            move_ordering.scored[i].1 = i;
        }

        move_ordering
    }

    fn in_pv_line(mv: &Move, pv_lines: &Vec<PVLine>, ply_from_root: u8) -> bool {
        for pv_line in pv_lines {
            if pv_line.get_move(ply_from_root as usize) == Some(mv) {
                return true;
            }
        }
        false
    }

    /// Returns the next move to be played with the next highest score
    /// Returns None if all moves have been played
    /// lazily picks the next move by using selection sort
    /// TODO might try to use quicksort to sort the moves completely
    pub fn pick_next_move(&mut self) -> Option<Move> {
        if self.cur_idx >= self.legal_moves.len() {
            return None;
        }

        let mut max_score = self.scored[self.cur_idx].0;
        let mut max_idx = 0;
        for i in (self.cur_idx + 1)..self.legal_moves.len() {
            if self.scored[i].0 > max_score {
                max_score = self.scored[i].0;
                max_idx = i;
            }
        }
        self.scored.swap(max_idx, self.cur_idx);
        self.cur_idx += 1;
        self.legal_moves.get(self.scored[self.cur_idx - 1].1)
    }
}

/// #### MVV-LVA (Most Valuable Victim - Least Valuable Aggressor) table  
/// The table is used to score captures
/// Indexed by MVV_LVA_TABLE\[**victim**]\[**attacker**]  
/// E.g. Pawn captures Queen -> MVV_LVA_TABLE\[**Queen**]\[**Pawn**] would be the highest score  
const MVV_LVA_TABLE: [[u16; 6]; 6] = [
    [60, 59, 58, 57, 56, 55],  // Pawn Victim, attacker P, N, B, R, Q, k
    [70, 69, 68, 67, 66, 65],  // Knight Victim, attacker P, N, B, R, Q, k
    [80, 79, 78, 77, 76, 75],  // Bishop Victim, attacker P, N, B, R, Q, k
    [90, 89, 88, 87, 86, 85],  // Rook Victim, attacker P, N, B, R, Q, k
    [100, 99, 98, 97, 96, 95], // Queen Victim, attacker P, N, B, R, Q, k
    [0, 0, 0, 0, 0, 0], // King Victim, attacker P, N, B, R, Q, k // King captures can't happen
];
