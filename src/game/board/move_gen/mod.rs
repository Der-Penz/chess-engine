mod helper;
pub mod magic;
mod test;
use attack_pattern::direction_mask::ALIGN_MASKS;
use helper::{MoveGenerationData, MoveGenerationMasks};

use crate::game::{
    bit_manipulation::drop_lsb,
    castle_rights::{CastleRights, CastleType},
    color::Color,
    move_notation::{Move, MoveFlag},
    piece_type::PieceType,
    square::Square,
};

use super::{bit_board::BitBoard, Board};

pub mod attack_pattern;

pub struct MoveGeneration();

impl MoveGeneration {
    pub fn generate_legal_moves(board: &Board) -> MoveList {
        let mut legal_moves = MoveList::default();

        let data = MoveGenerationData::from_board(&board);
        let mut masks = MoveGenerationMasks::default();

        masks.calculate_king_danger(&data, &board);

        //only king moves are allowed if in multi check (no other moves are allowed or castling)
        let king_moves = Self::attacks_king(data.king_sq, data.ally) & !masks.king_danger;

        legal_moves.create_and_add_moves(data.king_sq, king_moves, MoveFlag::Normal);
        if masks.multi_check {
            return legal_moves;
        }

        masks.calculate_pins(&data, board);
        masks.calculate_push_and_capture(&data, board);

        //calculate moves for pinned pieces
        if !masks.in_check {
            for sq in BitBoard::from(masks.orthogonal_pinned).get_occupied() {
                let piece = board.get_sq_piece(sq).expect("Pinned piece must exist");
                let pin_move_mask = ALIGN_MASKS[sq][data.king_sq];

                match piece.ptype() {
                    PieceType::Pawn => {
                        let double_push =
                            Self::moves_pawn_double_push(sq, data.enemy, data.ally, data.color);
                        legal_moves.create_and_add_moves(
                            sq,
                            double_push & pin_move_mask,
                            MoveFlag::DoublePawnPush,
                        );
                        let pawn_move = Self::moves_pawn(sq, data.enemy, data.ally, data.color);
                        add_pawn_moves(&mut legal_moves, sq, pawn_move & pin_move_mask, data.color);
                    }
                    //if queen is straight pinned, it can only move straight
                    PieceType::Rook | PieceType::Queen => {
                        let moves = Self::attacks_rook(sq, data.enemy, data.ally);
                        legal_moves.create_and_add_moves(
                            sq,
                            moves & pin_move_mask,
                            MoveFlag::Normal,
                        );
                    }
                    PieceType::Knight | PieceType::Bishop => (), //can't move if pinned by a rook like piece
                    PieceType::King => panic!("King can't be pinned"),
                };
            }
            for sq in BitBoard::from(masks.diagonal_pinned).get_occupied() {
                let piece = board.get_sq_piece(sq).expect("Pinned piece must exist");
                let pin_move_mask = ALIGN_MASKS[sq][data.king_sq];

                match piece.ptype() {
                    PieceType::Pawn => {
                        let en_passant = Self::attacks_pawn_en_passant(
                            sq,
                            data.color,
                            board.cur_state().en_passant.as_ref(),
                            data.king_sq,
                            data.enemy,
                            data.ally,
                            board,
                        );
                        legal_moves.create_and_add_moves(
                            sq,
                            en_passant & pin_move_mask,
                            MoveFlag::EnPassant,
                        );
                        let attacks = Self::attacks_pawn(sq, data.enemy, data.ally, data.color);
                        add_pawn_moves(&mut legal_moves, sq, attacks & pin_move_mask, data.color);
                    }
                    //if queen is diagonal pinned, it can only move diagonally
                    PieceType::Bishop | PieceType::Queen => {
                        let moves = Self::attacks_bishop(sq, data.enemy, data.ally);
                        legal_moves.create_and_add_moves(
                            sq,
                            moves & pin_move_mask,
                            MoveFlag::Normal,
                        );
                    }
                    PieceType::Knight | PieceType::Rook => (), //can't move if pinned by a bishop like piece
                    PieceType::King => panic!("King can't be pinned"),
                };
            }
        }

        //calculate moves for non-pinned pieces
        let non_pinned = data.ally & !(masks.diagonal_pinned | masks.orthogonal_pinned);
        for sq in BitBoard::from(non_pinned).get_occupied() {
            let piece = board
                .get_sq_piece(sq)
                .expect(format!("Piece at {} must exist", sq).as_str());

            match piece.ptype() {
                PieceType::Pawn => {
                    let pawn_move = Self::moves_pawn(sq, data.enemy, data.ally, data.color);
                    add_pawn_moves(
                        &mut legal_moves,
                        sq,
                        pawn_move & masks.push_capture_mask,
                        data.color,
                    );

                    let attacks = Self::attacks_pawn(sq, data.enemy, data.ally, data.color);
                    add_pawn_moves(
                        &mut legal_moves,
                        sq,
                        attacks & masks.push_capture_mask,
                        data.color,
                    );

                    let double_push =
                        Self::moves_pawn_double_push(sq, data.enemy, data.ally, data.color);
                    legal_moves.create_and_add_moves(
                        sq,
                        double_push & masks.push_capture_mask,
                        MoveFlag::DoublePawnPush,
                    );
                    let en_passant = Self::attacks_pawn_en_passant(
                        sq,
                        data.color,
                        board.cur_state().en_passant.as_ref(),
                        data.king_sq,
                        data.enemy,
                        data.ally,
                        board,
                    );
                    legal_moves.create_and_add_moves(
                        sq,
                        en_passant & masks.push_mask,
                        MoveFlag::EnPassant,
                    );
                    if en_passant != 0 {
                        let en_passant_pawn = board.cur_state().en_passant.unwrap().square_value()
                            as i8
                            - data.color.perspective() * 8;

                        if masks.checkers & (1u64 << en_passant_pawn) != 0 {
                            legal_moves.create_and_add_moves(sq, en_passant, MoveFlag::EnPassant);
                        }
                    }
                }
                PieceType::Knight => {
                    let moves = Self::attacks_knight(sq, data.ally);
                    legal_moves.create_and_add_moves(
                        sq,
                        moves & masks.push_capture_mask,
                        MoveFlag::Normal,
                    );
                }
                PieceType::Bishop => {
                    let moves = Self::attacks_bishop(sq, data.enemy, data.ally);
                    legal_moves.create_and_add_moves(
                        sq,
                        moves & masks.push_capture_mask,
                        MoveFlag::Normal,
                    );
                }
                PieceType::Rook => {
                    let moves = Self::attacks_rook(sq, data.enemy, data.ally);
                    legal_moves.create_and_add_moves(
                        sq,
                        moves & masks.push_capture_mask,
                        MoveFlag::Normal,
                    );
                }
                PieceType::Queen => {
                    let moves = Self::attacks_queen(sq, data.enemy, data.ally);
                    legal_moves.create_and_add_moves(
                        sq,
                        moves & masks.push_capture_mask,
                        MoveFlag::Normal,
                    );
                }
                PieceType::King => {
                    if !masks.in_check {
                        if board
                            .cur_state()
                            .castling_rights
                            .has(data.color, CastleType::KingSide)
                        {
                            legal_moves.create_and_add_moves(
                                sq,
                                Self::moves_king_castle_king_side(
                                    sq,
                                    data.enemy,
                                    data.ally,
                                    masks.king_danger,
                                    data.color,
                                ),
                                MoveFlag::KingSideCastle,
                            );
                        }
                        if board
                            .cur_state()
                            .castling_rights
                            .has(data.color, CastleType::QueenSide)
                        {
                            legal_moves.create_and_add_moves(
                                sq,
                                Self::moves_king_castle_queen_side(
                                    sq,
                                    data.enemy,
                                    data.ally,
                                    masks.king_danger,
                                    data.color,
                                ),
                                MoveFlag::QueenSideCastle,
                            );
                        }
                    }
                }
            }
        }

        legal_moves
    }

    #[inline(always)]
    pub fn attacks_rook(sq: Square, enemy: u64, ally: u64) -> u64 {
        let mut attacks = 0;
        attacks |= attack_pattern::rook_attacks_vertical(enemy, ally, sq);
        attacks |= attack_pattern::rook_attacks_horizontal(enemy, ally, sq);
        attacks
    }

    #[inline(always)]
    pub fn attacks_bishop(sq: Square, enemy: u64, ally: u64) -> u64 {
        let mut attacks = 0;
        attacks |= attack_pattern::bishop_attacks_main(enemy, ally, sq);
        attacks |= attack_pattern::bishop_attacks_anti(enemy, ally, sq);
        attacks
    }

    #[inline(always)]
    pub fn attacks_queen(sq: Square, enemy: u64, ally: u64) -> u64 {
        let mut attacks = 0;
        attacks |= Self::attacks_rook(sq, enemy, ally);
        attacks |= Self::attacks_bishop(sq, enemy, ally);
        attacks
    }

    #[inline(always)]
    pub fn attacks_knight(sq: Square, ally: u64) -> u64 {
        attack_pattern::ATTACK_PATTERN_KNIGHT[sq] & !ally
    }

    #[inline(always)]
    pub fn attacks_king(sq: Square, ally: u64) -> u64 {
        attack_pattern::ATTACK_PATTERN_KING[sq] & !ally
    }

    pub fn moves_king_castle_queen_side(
        sq: Square,
        enemy: u64,
        ally: u64,
        attacked: u64,
        color: Color,
    ) -> u64 {
        if sq != CastleType::KING_SOURCE[color] {
            return 0;
        }

        let all = ally | enemy;
        let queen_side_free = attack_pattern::CASTLE_FREE_SQUARES[color][CastleType::QueenSide];
        let queen_side_attack_free =
            attack_pattern::CASTLE_ATTACK_FREE_SQUARES[color][CastleType::QueenSide];

        let queen_side_possible =
            (queen_side_free & all) == 0 && (queen_side_attack_free & attacked) == 0;

        if queen_side_possible {
            CastleType::KING_DEST[CastleRights::to_index(color, CastleType::QueenSide) as usize]
                .to_mask()
        } else {
            0
        }
    }

    pub fn moves_king_castle_king_side(
        sq: Square,
        enemy: u64,
        ally: u64,
        attacked: u64,
        color: Color,
    ) -> u64 {
        if sq != CastleType::KING_SOURCE[color] {
            return 0;
        }

        let all = ally | enemy;
        let king_side_free = attack_pattern::CASTLE_FREE_SQUARES[color][CastleType::KingSide];
        let king_side_attack_free =
            attack_pattern::CASTLE_ATTACK_FREE_SQUARES[color][CastleType::KingSide];

        let king_side_possible =
            (king_side_free & all) == 0 && (king_side_attack_free & attacked) == 0;

        if king_side_possible {
            CastleType::KING_DEST[CastleRights::to_index(color, CastleType::KingSide) as usize]
                .to_mask()
        } else {
            0
        }
    }

    #[inline(always)]
    pub fn attacks_pawn(sq: Square, enemy: u64, ally: u64, color: Color) -> u64 {
        (attack_pattern::ATTACK_PATTERN_PAWN[color][sq] & !ally) & enemy
    }

    #[inline(always)]
    pub fn moves_pawn_double_push(sq: Square, enemy: u64, ally: u64, color: Color) -> u64 {
        if sq.rank() != color.pawn_rank() {
            return 0;
        }
        let index = (sq.square_value() as i8 + (color.perspective() * 8)) as u8;
        let all = BitBoard::new(ally | enemy);
        if all.is_occupied(Square::new(index)) {
            return 0;
        }

        attack_pattern::MOVE_PATTERN_PAWN[color][index as usize] & !*all
    }

    #[inline(always)]
    pub fn attacks_pawn_en_passant(
        sq: Square,
        color: Color,
        en_passant: Option<&Square>,
        king_sq: Square,
        enemy: u64,
        ally: u64,
        board: &Board,
    ) -> u64 {
        if en_passant.is_none() {
            return 0;
        }
        let en_passant_square = en_passant.unwrap();

        let enemy_pawn_mask =
            1u64 << (en_passant_square.square_value() as i8 - (color.perspective() * 8));
        let sq_mask = sq.to_mask();

        //check for if the en passant capture would expose a discovered attack on the king
        //by removing the pawn and the en passant pawn from the board, we can check if there
        //is a rook or queen attacking the king by using the rook horizontal attack pattern from the king square
        //and check if this ray would attack a enemy rook or queen
        let enemy_without_pawn = enemy & !(enemy_pawn_mask);
        let ally_without_pawn = ally & !(sq_mask);
        let rank_attack_ray =
            attack_pattern::rook_attacks_horizontal(enemy_without_pawn, ally_without_pawn, king_sq);

        if king_sq.rank() == sq.rank() && rank_attack_ray & enemy_without_pawn != 0 {
            let horizontal_rook_attack = BitBoard::from(rank_attack_ray & enemy_without_pawn)
                .get_occupied()
                .any(|square| {
                    let piece = board.get_sq_piece(square).map(|piece| piece.ptype());

                    piece == Some(PieceType::Rook) || piece == Some(PieceType::Queen)
                });

            //if there is a rook or queen attacking the king, the en passant is invalid
            if horizontal_rook_attack {
                return 0;
            }
        }

        attack_pattern::ATTACK_PATTERN_PAWN[color][sq] & en_passant_square.to_mask()
    }

    #[inline(always)]
    pub fn moves_pawn(sq: Square, enemy: u64, ally: u64, color: Color) -> u64 {
        attack_pattern::MOVE_PATTERN_PAWN[color][sq] & !(ally | enemy)
    }
}

/// returns a mask of squares between two squares in vertical or horizontal direction
/// If the squares are not in the same rank or file, the mask will be invalid and should not be used
fn sq_betweens_rook_rays(first: Square, second: Square) -> u64 {
    let first_mask = first.to_mask();
    let second_mask = second.to_mask();
    let mut ray_first = attack_pattern::rook_attacks_vertical(0, second_mask, first);
    ray_first |= attack_pattern::rook_attacks_horizontal(0, second_mask, first);
    let mut ray_second = attack_pattern::rook_attacks_vertical(0, first_mask, second);
    ray_second |= attack_pattern::rook_attacks_horizontal(0, first_mask, second);
    ray_first & ray_second
}

/// returns a mask of squares between two squares in diagonal or anti-diagonal direction
/// If the squares are not in the same rank or file, the mask will be invalid and should not be used
fn sq_betweens_bishop_rays(first: Square, second: Square) -> u64 {
    let first_mask = first.to_mask();
    let second_mask = second.to_mask();
    let mut ray_first = attack_pattern::bishop_attacks_main(0, second_mask, first);
    ray_first |= attack_pattern::bishop_attacks_anti(0, second_mask, first);
    let mut ray_second = attack_pattern::bishop_attacks_main(0, first_mask, second);
    ray_second |= attack_pattern::bishop_attacks_anti(0, first_mask, second);
    ray_first & ray_second
}

/// Adds pawn moves to the move list for a given source and destination square
/// handles promotions as well
fn add_pawn_moves(moves: &mut MoveList, source: Square, dest: u64, color: Color) {
    BitBoard::from(dest).get_occupied().for_each(|dest| {
        if dest.rank() != color.promotion_rank() {
            moves.add_move(Move::new(source, dest, MoveFlag::Normal));
        } else {
            moves.add_move(Move::new(source, dest, MoveFlag::QueenPromotion));
            moves.add_move(Move::new(source, dest, MoveFlag::RookPromotion));
            moves.add_move(Move::new(source, dest, MoveFlag::BishopPromotion));
            moves.add_move(Move::new(source, dest, MoveFlag::KnightPromotion));
        }
    });
}

const MAX_NUMBER_OF_MOVES_PER_POSITION: usize = 218;
type MoveListArray = [Move; MAX_NUMBER_OF_MOVES_PER_POSITION];

#[derive(Debug, Clone, Copy)]
pub struct MoveList {
    moves: MoveListArray,
    count: usize,
}

impl std::default::Default for MoveList {
    fn default() -> Self {
        Self {
            moves: [Move::default(); MAX_NUMBER_OF_MOVES_PER_POSITION],
            count: 0,
        }
    }
}

impl MoveList {
    pub fn add_move(&mut self, m: Move) {
        self.moves[self.count] = m;
        self.count += 1;
    }

    pub fn create_and_add_moves(&mut self, source: Square, moves: u64, flag: MoveFlag) {
        let mut moves = moves;
        loop {
            if moves == 0 {
                break;
            }

            let dest = drop_lsb(&mut moves);
            self.add_move(Move::new(source, dest, flag));
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn get(&self, index: usize) -> Option<&Move> {
        self.moves.get(index)
    }

    pub fn has(&self, m: &Move) -> bool {
        self.moves[..self.count].contains(m)
    }

    pub fn as_vec(&self) -> Vec<Move> {
        self.moves[..self.count].to_vec()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Move> {
        self.moves[..self.count].iter()
    }

    pub fn as_attack_bb(&self) -> BitBoard {
        let mut bb = BitBoard::default();
        for m in self.iter() {
            bb.set(m.dest());
        }
        bb
    }
}

impl std::fmt::Display for MoveList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Moves: ")?;
        for m in self.iter() {
            write!(f, "{}, ", m)?;
        }
        Ok(())
    }
}
