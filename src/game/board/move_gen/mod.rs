mod test;
use attack_pattern::direction_mask::ALIGN_MASKS;

use crate::game::{
    bit_manipulation::{north_east, north_west, south_east, south_west},
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

        let color = board.side_to_move();
        let ally = *board.get_bb_occupied(color);
        let enemy = *board.get_bb_occupied(color.opposite());
        let bb_king = *board.get_bb_for(PieceType::King.as_colored_piece(color));
        let king_sq = board.get_king_pos(color);

        let king_danger_squares =
            Self::generate_king_danger_squares(ally, enemy, bb_king, color, &board);

        let in_check = (king_danger_squares & king_sq.to_mask()) != 0;

        //calculate checkers
        let checkers = if !in_check {
            0
        } else {
            Self::generate_checkers(king_sq, color, board)
        };

        let multi_check = checkers.count_ones() > 1;

        //only king moves are allowed if in multi check (no other moves are allowed or castling)
        let king_moves = Self::attacks_king(king_sq, ally) & !king_danger_squares;
        legal_moves.create_and_add_moves(king_sq, king_moves, MoveFlag::Normal);
        if multi_check {
            return legal_moves;
        }

        let (_pin_move_mask, straight_pinned_pieces, diagonal_pinned_pieces) =
            Self::generate_pins(king_sq, ally, enemy, color.opposite(), board);
        let (push_mask, capture_mask) =
            Self::generate_push_and_capture_mask(in_check, checkers, king_sq, board);

        //calculate moves for pinned pieces
        if !in_check {
            for sq in BitBoard::from(straight_pinned_pieces).get_occupied() {
                let piece = board.get_sq_piece(sq).expect("Pinned piece must exist");
                let pin_move_mask =
                    ALIGN_MASKS[sq.square_value() as usize][king_sq.square_value() as usize];

                match piece.ptype() {
                    PieceType::Pawn => {
                        let double_push = Self::moves_pawn_double_push(sq, enemy, ally, color);
                        legal_moves.create_and_add_moves(
                            sq,
                            double_push & pin_move_mask,
                            MoveFlag::DoublePawnPush,
                        );
                        let pawn_move = Self::moves_pawn(sq, enemy, ally, color);
                        add_pawn_moves(&mut legal_moves, sq, pawn_move & pin_move_mask, color);
                    }
                    //if queen is straight pinned, it can only move straight
                    PieceType::Rook | PieceType::Queen => {
                        let moves = Self::attacks_rook(sq, enemy, ally);
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
            for sq in BitBoard::from(diagonal_pinned_pieces).get_occupied() {
                let piece = board.get_sq_piece(sq).expect("Pinned piece must exist");
                let pin_move_mask =
                    ALIGN_MASKS[sq.square_value() as usize][king_sq.square_value() as usize];

                match piece.ptype() {
                    PieceType::Pawn => {
                        let en_passant = Self::attacks_pawn_en_passant(
                            sq,
                            color,
                            board.cur_state().en_passant.as_ref(),
                            king_sq,
                            enemy,
                            ally,
                            board,
                        );
                        legal_moves.create_and_add_moves(
                            sq,
                            en_passant & pin_move_mask,
                            MoveFlag::EnPassant,
                        );
                        let attacks = Self::attacks_pawn(sq, enemy, ally, color);
                        add_pawn_moves(&mut legal_moves, sq, attacks & pin_move_mask, color);
                    }
                    //if queen is diagonal pinned, it can only move diagonally
                    PieceType::Bishop | PieceType::Queen => {
                        let moves = Self::attacks_bishop(sq, enemy, ally);
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
        let non_pinned = ally & !(straight_pinned_pieces | diagonal_pinned_pieces);

        for sq in BitBoard::from(non_pinned).get_occupied() {
            let piece = board
                .get_sq_piece(sq)
                .expect(format!("Piece at {} must exist", sq).as_str());

            match piece.ptype() {
                PieceType::Pawn => {
                    let pawn_move = Self::moves_pawn(sq, enemy, ally, color);
                    add_pawn_moves(
                        &mut legal_moves,
                        sq,
                        pawn_move & (push_mask | capture_mask),
                        color,
                    );

                    let attacks = Self::attacks_pawn(sq, enemy, ally, color);
                    add_pawn_moves(
                        &mut legal_moves,
                        sq,
                        attacks & (push_mask | capture_mask),
                        color,
                    );

                    let double_push = Self::moves_pawn_double_push(sq, enemy, ally, color);
                    legal_moves.create_and_add_moves(
                        sq,
                        double_push & (push_mask | capture_mask),
                        MoveFlag::DoublePawnPush,
                    );
                    let en_passant = Self::attacks_pawn_en_passant(
                        sq,
                        color,
                        board.cur_state().en_passant.as_ref(),
                        king_sq,
                        enemy,
                        ally,
                        board,
                    );
                    legal_moves.create_and_add_moves(
                        sq,
                        en_passant & push_mask,
                        MoveFlag::EnPassant,
                    );
                    if en_passant != 0 {
                        let en_passant_pawn = board.cur_state().en_passant.unwrap().square_value()
                            as i8
                            - color.perspective() * 8;

                        if checkers & (1u64 << en_passant_pawn) != 0 {
                            legal_moves.create_and_add_moves(sq, en_passant, MoveFlag::EnPassant);
                        }
                    }
                }
                PieceType::Knight => {
                    let moves = Self::attacks_knight(sq, ally);
                    legal_moves.create_and_add_moves(
                        sq,
                        moves & (push_mask | capture_mask),
                        MoveFlag::Normal,
                    );
                }
                PieceType::Bishop => {
                    let moves = Self::attacks_bishop(sq, enemy, ally);
                    legal_moves.create_and_add_moves(
                        sq,
                        moves & (push_mask | capture_mask),
                        MoveFlag::Normal,
                    );
                }
                PieceType::Rook => {
                    let moves = Self::attacks_rook(sq, enemy, ally);
                    legal_moves.create_and_add_moves(
                        sq,
                        moves & (push_mask | capture_mask),
                        MoveFlag::Normal,
                    );
                }
                PieceType::Queen => {
                    let moves = Self::attacks_queen(sq, enemy, ally);
                    legal_moves.create_and_add_moves(
                        sq,
                        moves & (push_mask | capture_mask),
                        MoveFlag::Normal,
                    );
                }
                PieceType::King => {
                    if !in_check {
                        if board
                            .cur_state()
                            .castling_rights
                            .has(color, CastleType::KingSide)
                        {
                            legal_moves.create_and_add_moves(
                                sq,
                                Self::moves_king_castle_king_side(
                                    sq,
                                    enemy,
                                    ally,
                                    king_danger_squares,
                                    color,
                                ),
                                MoveFlag::KingSideCastle,
                            );
                        }
                        if board
                            .cur_state()
                            .castling_rights
                            .has(color, CastleType::QueenSide)
                        {
                            legal_moves.create_and_add_moves(
                                sq,
                                Self::moves_king_castle_queen_side(
                                    sq,
                                    enemy,
                                    ally,
                                    king_danger_squares,
                                    color,
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

    fn generate_push_and_capture_mask(
        in_check: bool,
        checkers: u64,
        king_sq: Square,
        board: &Board,
    ) -> (u64, u64) {
        let mut capture_mask = 0xFFFFFFFFFFFFFFFFu64;
        let mut push_mask = 0xFFFFFFFFFFFFFFFFu64;

        //if only one check is present, we can capture the checking piece or block the check
        if in_check {
            capture_mask = checkers;

            let check_sq = BitBoard::from(checkers)
                .get_occupied()
                .next()
                .expect("Checkers board must not be empty if in check");
            let check_piece = board
                .get_sq_piece(check_sq)
                .expect("Checker piece must be present if in check");

            //check can only be blocked if it is a sliding piece
            if check_piece.ptype().is_sliding_piece() {
                //calculate the squares between the king and the checker
                let same_file_or_rank =
                    check_sq.file() == king_sq.file() || check_sq.rank() == king_sq.rank();

                push_mask = if same_file_or_rank {
                    sq_betweens_rook_rays(check_sq, king_sq)
                } else {
                    sq_betweens_bishop_rays(check_sq, king_sq)
                };
            } else {
                push_mask = 0;
            }
        }
        (push_mask, capture_mask)
    }

    fn generate_pins(
        king_sq: Square,
        ally: u64,
        enemy: u64,
        enemy_color: Color,
        board: &Board,
    ) -> (u64, u64, u64) {
        let mut pin_move_mask = 0u64;
        let mut straight_pinned_pieces = 0u64;
        let mut diagonal_pinned_pieces = 0u64;

        let king_mask = king_sq.to_mask();

        //TODO could be faster if I use a table with the directions and & those to (king in direction) & (slider in opposite direction)
        for sq in board.get_bb_rook_slider(enemy_color).get_occupied() {
            let slider_mask = sq.to_mask();
            let slider_vertical = attack_pattern::rook_attacks_vertical(0, king_mask, sq);
            let slider_horizontal = attack_pattern::rook_attacks_horizontal(0, king_mask, sq);
            let king_vertical = attack_pattern::rook_attacks_vertical(0, slider_mask, king_sq);
            let king_horizontal = attack_pattern::rook_attacks_horizontal(0, slider_mask, king_sq);
            let ray = king_vertical & slider_vertical;
            if ray != 0 {
                if (ray & ally).count_ones() == 1 && ray & enemy == 0 {
                    straight_pinned_pieces |= ray & ally;
                }
                pin_move_mask |= ray | slider_mask;
            }
            let ray = king_horizontal & slider_horizontal;
            if ray != 0 {
                if (ray & ally).count_ones() == 1 && ray & enemy == 0 {
                    straight_pinned_pieces |= ray & ally;
                }
                pin_move_mask |= ray | slider_mask;
            }
        }

        for sq in board.get_bb_bishop_slider(enemy_color).get_occupied() {
            let slider_mask = sq.to_mask();
            let slider_main = attack_pattern::bishop_attacks_main(0, king_mask, sq);
            let slider_anti = attack_pattern::bishop_attacks_anti(0, king_mask, sq);
            let king_main = attack_pattern::bishop_attacks_main(0, slider_mask, king_sq);
            let king_anti = attack_pattern::bishop_attacks_anti(0, slider_mask, king_sq);
            let ray = king_main & slider_main;
            if ray != 0 {
                if (ray & ally).count_ones() == 1 && ray & enemy == 0 {
                    diagonal_pinned_pieces |= ray & ally;
                }
                pin_move_mask |= ray | slider_mask;
            }
            let ray = king_anti & slider_anti;
            if ray != 0 {
                if (ray & ally).count_ones() == 1 && ray & enemy == 0 {
                    diagonal_pinned_pieces |= ray & ally;
                }
                pin_move_mask |= ray | slider_mask;
            }
        }

        (
            pin_move_mask,
            straight_pinned_pieces,
            diagonal_pinned_pieces,
        )
    }

    fn generate_king_danger_squares(
        bb_allies: u64,
        bb_enemies: u64,
        bb_king: u64,
        color: Color,
        board: &Board,
    ) -> u64 {
        let mut danger_squares = 0u64;

        let enemy_color = color.opposite();

        //King attacks
        danger_squares |= Self::attacks_king(board.get_king_pos(enemy_color), 0);

        //Pawn attacks
        let bb_pawns = *board.get_bb_for(PieceType::Pawn.as_colored_piece(enemy_color));
        if enemy_color == Color::White {
            danger_squares |= north_east(bb_pawns, 1);
            danger_squares |= north_west(bb_pawns, 1);
        } else {
            danger_squares |= south_east(bb_pawns, 1);
            danger_squares |= south_west(bb_pawns, 1);
        }

        //Knight attacks
        for knight_sq in board.get_piece_positions(PieceType::Knight.as_colored_piece(enemy_color))
        {
            danger_squares |= Self::attacks_knight(knight_sq, 0);
        }

        //skip more expensive slider calculations if there are no sliders
        if *board.bb_sliders[enemy_color] == 0 {
            return danger_squares;
        }

        let bb_allies_without_king = bb_allies & !bb_king;

        //Sliding attacks
        for sq in board.get_piece_positions(PieceType::Bishop.as_colored_piece(enemy_color)) {
            danger_squares |= Self::attacks_bishop(sq, bb_enemies, bb_allies_without_king);
        }
        for sq in board.get_piece_positions(PieceType::Rook.as_colored_piece(enemy_color)) {
            danger_squares |= Self::attacks_rook(sq, bb_enemies, bb_allies_without_king);
        }
        for sq in board.get_piece_positions(PieceType::Queen.as_colored_piece(enemy_color)) {
            danger_squares |= Self::attacks_bishop(sq, bb_enemies, bb_allies_without_king);
            danger_squares |= Self::attacks_rook(sq, bb_enemies, bb_allies_without_king);
        }

        danger_squares
    }

    /// Generates the checkers BB for a given square and color. Square is the king square. Checkers are from the opposite color
    fn generate_checkers(king_pos: Square, color: Color, board: &Board) -> u64 {
        let attack_color = color.opposite();
        let ally = *board.get_bb_occupied(color);
        let enemy = *board.get_bb_occupied(attack_color);
        let enemy_bb = board.get_bb_pieces()[attack_color];

        let mut checkers = 0u64;

        //check for attacks from non-sliding pieces
        //no need to check for king attacks, as a king can't attack another king
        checkers |= MoveGeneration::attacks_knight(king_pos, ally) & *enemy_bb[PieceType::Knight];
        checkers |=
            MoveGeneration::attacks_pawn(king_pos, enemy, ally, color) & *enemy_bb[PieceType::Pawn];

        //if no sliding pieces are available, we won't need to check for attacks
        if *board.bb_sliders[attack_color] == 0 {
            return checkers;
        }

        checkers |= MoveGeneration::attacks_bishop(king_pos, enemy, ally)
            & (*enemy_bb[PieceType::Bishop] | *enemy_bb[PieceType::Queen]);
        checkers |= MoveGeneration::attacks_rook(king_pos, enemy, ally)
            & (*enemy_bb[PieceType::Rook] | *enemy_bb[PieceType::Queen]);

        checkers
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
                    board
                        .get_sq_piece(square)
                        .map(|piece| matches!(piece.ptype(), PieceType::Rook | PieceType::Queen))
                        .is_some()
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
        BitBoard::from(moves).get_occupied().for_each(|dest| {
            self.add_move(Move::new(source, dest, flag));
        });
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
