use crate::game::{
    bit_manipulation::drop_lsb, board::bit_board::BitBoard, move_notation::MoveFlag, Color, Move,
    Square,
};

use super::helper::MoveGenerationMasks;

const MAX_NUMBER_OF_MOVES_PER_POSITION: usize = 218;
type MoveListArray = [Move; MAX_NUMBER_OF_MOVES_PER_POSITION];

#[derive(Debug)]
pub struct LegalMoveList {
    moves: MoveListArray,
    masks: MoveGenerationMasks,
    count: usize,
}

impl std::default::Default for LegalMoveList {
    fn default() -> Self {
        Self {
            moves: [Move::default(); MAX_NUMBER_OF_MOVES_PER_POSITION],
            masks: MoveGenerationMasks::default(),
            count: 0,
        }
    }
}

impl LegalMoveList {
    ///Adds a single move to the move list
    ///If the move list is full, the move is not added
    #[inline]
    fn add_move(&mut self, m: Move) {
        if self.count == MAX_NUMBER_OF_MOVES_PER_POSITION {
            return;
        }

        self.moves[self.count] = m;
        self.count += 1;
    }

    /// Adds multiple moves to the move list from a given source square with a given flag
    pub(super) fn create_and_add_moves(
        &mut self,
        source: Square,
        moves: u64,
        flag: MoveFlag,
        move_type_mask: u64,
    ) {
        let mut moves = moves & move_type_mask;
        loop {
            if moves == 0 {
                break;
            }

            let dest = drop_lsb(&mut moves);
            self.add_move(Move::new(source, dest, flag));
        }
    }

    /// Adds pawn moves to the move list for a given source and destination square
    /// handles promotions as well
    pub(super) fn create_and_add_pawn_moves(
        &mut self,
        source: Square,
        dest: u64,
        color: Color,
        move_type_mask: u64,
    ) {
        let dest = dest & move_type_mask;

        BitBoard::from(dest).get_occupied().for_each(|dest| {
            if dest.rank() != color.promotion_rank() {
                self.add_move(Move::new(source, dest, MoveFlag::Normal));
            } else {
                self.add_move(Move::new(source, dest, MoveFlag::QueenPromotion));
                self.add_move(Move::new(source, dest, MoveFlag::RookPromotion));
                self.add_move(Move::new(source, dest, MoveFlag::BishopPromotion));
                self.add_move(Move::new(source, dest, MoveFlag::KnightPromotion));
            }
        });
    }

    #[inline(always)]
    pub fn is_checkmate(&self) -> bool {
        self.count == 0 && self.masks.in_check
    }

    #[inline(always)]
    pub fn is_stalemate(&self) -> bool {
        self.count == 0 && !self.masks.in_check
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn get(&self, index: usize) -> Option<Move> {
        self.moves[..self.count].get(index).copied()
    }

    pub fn has(&self, m: &Move) -> bool {
        self.moves[..self.count].contains(m)
    }

    pub fn as_vec(&self) -> Vec<Move> {
        self.moves[..self.count].to_vec()
    }

    pub fn get_all(&self) -> &[Move] {
        &self.moves[..self.count]
    }

    pub fn iter(&self) -> impl Iterator<Item = &Move> {
        self.moves[..self.count].iter()
    }

    ///Consumes the move list and returns a move list from the given move list with only captures
    pub fn to_captures_only(self) -> LegalMoveList {
        let mut captures_only = LegalMoveList::default();

        captures_only.set_masks(self.masks);
        self.moves[..self.count]
            .iter()
            .filter(|m| m.dest().to_mask() & self.masks.enemy != 0)
            .for_each(|m| {
                captures_only.add_move(*m);
            });

        captures_only
    }

    pub fn as_attack_bb(&self) -> BitBoard {
        let mut bb = BitBoard::default();
        for m in self.iter() {
            bb.set(m.dest());
        }
        bb
    }

    pub fn get_masks(&self) -> &MoveGenerationMasks {
        &self.masks
    }

    pub(super) fn set_masks(&mut self, masks: MoveGenerationMasks) {
        self.masks = masks;
    }
}

impl std::fmt::Display for LegalMoveList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} Moves: ", self.count)?;
        for m in self.iter() {
            write!(f, "{}, ", m)?;
        }
        Ok(())
    }
}
