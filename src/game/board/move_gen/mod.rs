use crate::game::{
    bit_manipulation::{north_east, north_west, south_east, south_west},
    board::display::BoardDisplay,
    castle_rights::{CastleRights, CastleType},
    color::Color,
    move_notation::{Move, MoveFlag},
    piece::Piece,
    piece_type::PieceType,
    square::Square,
};

use super::{bit_board::BitBoard, board_state::BoardState, Board};

pub mod attack_pattern;

pub struct MoveGeneration();

const MAX_NUMBER_OF_MOVES_PER_POSITION: usize = 218;

impl MoveGeneration {
    fn add_moves(
        moves: u64,
        from: Square,
        flag: MoveFlag,
        legal_moves: &mut [Move; 218],
        count: &mut usize,
    ) {
        BitBoard::from(moves).get_occupied().for_each(|sq| {
            legal_moves[*count] = Move::new(from, sq, flag);
            *count += 1;
        });
    }

    pub fn generate_legal_moves(
        board: &Board,
    ) -> ([Move; MAX_NUMBER_OF_MOVES_PER_POSITION], usize) {
        let mut legal_moves = [Move::default(); MAX_NUMBER_OF_MOVES_PER_POSITION];
        let mut count = 0;

        let color = board.side_to_move();

        let ally = **board.get_bb_occupied(&color);
        let enemy = **board.get_bb_occupied(&color.opposite());
        let bb_king = **board.get_bb_for(&PieceType::King.as_colored_piece(color));
        let king_sq = board.get_king_pos(&color);

        let king_danger_squares =
            Self::generate_king_danger_squares(ally, enemy, bb_king, color, &board);

        let in_check = king_danger_squares.is_occupied(&king_sq);

        //calculate checkers
        let checkers = if !in_check {
            BitBoard::default()
        } else {
            Self::generate_checkers(king_sq, color, board)
        };

        let multi_check = checkers.count_ones() > 1;

        //only king moves are allowed if in multi check (no other moves are allowed or castling)
        let king_moves = Self::attacks_king(king_sq, ally) & !*king_danger_squares;
        Self::add_moves(
            king_moves,
            king_sq,
            MoveFlag::Normal,
            &mut legal_moves,
            &mut count,
        );
        if multi_check {
            return (legal_moves, count);
        }

        let (pin_move_mask, straight_pinned_pieces, diagonal_pinned_pieces) =
            Self::generate_pins(king_sq, ally, enemy, color.opposite(), board);
        let (push_mask, capture_mask) =
            Self::generate_push_and_capture_mask(in_check, checkers.into(), king_sq, board);

        println!(
            "Push Mask\n{}",
            BoardDisplay::as_board_with_attacks(&Board::empty(), push_mask.into())
        );
        println!(
            "Capture Mask\n{}",
            BoardDisplay::as_board_with_attacks(&Board::empty(), capture_mask.into())
        );
        println!(
            "Pin move\n{}",
            BoardDisplay::as_board_with_attacks(&Board::empty(), pin_move_mask.into())
        );
        println!(
            "Straight pinned pieces\n{}",
            BoardDisplay::as_board_with_attacks(&Board::empty(), straight_pinned_pieces.into())
        );
        println!(
            "Diagonal pinned pieces\n{}",
            BoardDisplay::as_board_with_attacks(&Board::empty(), diagonal_pinned_pieces.into())
        );
        println!(
            "King danger mask\n{}",
            BoardDisplay::as_board_with_attacks(&Board::empty(), king_danger_squares)
        );

        //calculate moves for pinned pieces
        for sq in BitBoard::from(straight_pinned_pieces).get_occupied() {
            let piece = board.get_sq_piece(&sq).expect("Pinned piece must exist");

            let moves = match piece.ptype() {
                PieceType::Pawn => {
                    let double_push = Self::moves_pawn_double_push(sq, enemy, ally, color);
                    Self::add_moves(
                        double_push & pin_move_mask,
                        sq,
                        MoveFlag::DoublePawnPush,
                        &mut legal_moves,
                        &mut count,
                    );
                    Self::moves_pawn(sq, enemy, ally, color)
                }
                PieceType::Rook => Self::attacks_rook(sq, enemy, ally),
                PieceType::Queen => Self::attacks_queen(sq, enemy, ally),
                PieceType::Knight | PieceType::Bishop => 0, //can't move if pinned by a rook like piece
                PieceType::King => panic!("King can't be pinned"),
            };

            Self::add_moves(
                moves & pin_move_mask,
                sq,
                MoveFlag::Normal,
                &mut legal_moves,
                &mut count,
            );
        }
        for sq in BitBoard::from(diagonal_pinned_pieces).get_occupied() {
            let piece = board.get_sq_piece(&sq).expect("Pinned piece must exist");

            let moves = match piece.ptype() {
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
                    Self::add_moves(
                        en_passant & pin_move_mask,
                        sq,
                        MoveFlag::EnPassant,
                        &mut legal_moves,
                        &mut count,
                    );
                    Self::attacks_pawn(sq, enemy, ally, color)
                }
                PieceType::Bishop => Self::attacks_bishop(sq, enemy, ally),
                PieceType::Queen => Self::attacks_queen(sq, enemy, ally),
                PieceType::Knight | PieceType::Rook => 0, //can't move if pinned by a bishop like piece
                PieceType::King => panic!("King can't be pinned"),
            };

            Self::add_moves(
                moves & pin_move_mask,
                sq,
                MoveFlag::Normal,
                &mut legal_moves,
                &mut count,
            );
        }

        //calculate moves for non-pinned pieces
        let non_pinned = ally & !(straight_pinned_pieces | diagonal_pinned_pieces);

        for sq in BitBoard::from(non_pinned).get_occupied() {
            let piece = board.get_sq_piece(&sq).expect("Piece must exist");

            match piece.ptype() {
                PieceType::Pawn => {
                    let mut moves = Self::moves_pawn(sq, enemy, ally, color);
                    moves |= Self::attacks_pawn(sq, enemy, ally, color);
                    Self::add_moves(
                        moves & (push_mask | capture_mask),
                        sq,
                        MoveFlag::Normal,
                        &mut legal_moves,
                        &mut count,
                    );
                    let double_push = Self::moves_pawn_double_push(sq, enemy, ally, color);
                    Self::add_moves(
                        double_push & (push_mask | capture_mask),
                        sq,
                        MoveFlag::Normal,
                        &mut legal_moves,
                        &mut count,
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
                    Self::add_moves(
                        en_passant & (push_mask | capture_mask),
                        sq,
                        MoveFlag::EnPassant,
                        &mut legal_moves,
                        &mut count,
                    );
                }
                PieceType::Knight => {
                    let moves = Self::attacks_knight(sq, ally);
                    Self::add_moves(
                        moves & (push_mask | capture_mask),
                        sq,
                        MoveFlag::Normal,
                        &mut legal_moves,
                        &mut count,
                    );
                }
                PieceType::Bishop => {
                    let moves = Self::attacks_bishop(sq, enemy, ally);
                    Self::add_moves(
                        moves & (push_mask | capture_mask),
                        sq,
                        MoveFlag::Normal,
                        &mut legal_moves,
                        &mut count,
                    );
                }
                PieceType::Rook => {
                    let moves = Self::attacks_rook(sq, enemy, ally);
                    Self::add_moves(
                        moves & (push_mask | capture_mask),
                        sq,
                        MoveFlag::Normal,
                        &mut legal_moves,
                        &mut count,
                    );
                }
                PieceType::Queen => {
                    let moves = Self::attacks_queen(sq, enemy, ally);
                    Self::add_moves(
                        moves & (push_mask | capture_mask),
                        sq,
                        MoveFlag::Normal,
                        &mut legal_moves,
                        &mut count,
                    );
                }
                PieceType::King => {
                    if !in_check {
                        //TODO king danger squares might be wrong since ray attacks go through the king
                        Self::add_moves(
                            Self::moves_king_castle_king_side(
                                sq,
                                enemy,
                                ally,
                                *king_danger_squares,
                                color,
                            ),
                            sq,
                            MoveFlag::KingSideCastle,
                            &mut legal_moves,
                            &mut count,
                        );
                        Self::add_moves(
                            Self::moves_king_castle_queen_side(
                                sq,
                                enemy,
                                ally,
                                *king_danger_squares,
                                color,
                            ),
                            sq,
                            MoveFlag::QueenSideCastle,
                            &mut legal_moves,
                            &mut count,
                        );
                    }
                }
            }
        }

        (legal_moves, count)
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
                .get_sq_piece(&check_sq)
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
    ) -> BitBoard {
        let mut danger_squares = 0u64;

        let enemy_color = color.opposite();

        //King attacks
        danger_squares |= Self::attacks_king(board.get_king_pos(&enemy_color), 0);

        //Pawn attacks
        let bb_pawns = **board.get_bb_for(&PieceType::Pawn.as_colored_piece(enemy_color));
        if enemy_color == Color::White {
            danger_squares |= north_east(bb_pawns, 1);
            danger_squares |= north_west(bb_pawns, 1);
        } else {
            danger_squares |= south_east(bb_pawns, 1);
            danger_squares |= south_west(bb_pawns, 1);
        }

        //Knight attacks
        for knight_sq in board.get_piece_positions(&PieceType::Knight.as_colored_piece(enemy_color))
        {
            danger_squares |= Self::attacks_knight(knight_sq, 0);
        }

        //skip more expensive slider calculations if there are no sliders
        if *board.bb_sliders[enemy_color] == 0 {
            return danger_squares.into();
        }

        let bb_allies_without_king = bb_allies & !bb_king;

        //Sliding attacks
        for sq in board.get_piece_positions(&PieceType::Bishop.as_colored_piece(enemy_color)) {
            danger_squares |= Self::attacks_bishop(sq, bb_enemies, bb_allies_without_king);
        }
        for sq in board.get_piece_positions(&PieceType::Rook.as_colored_piece(enemy_color)) {
            danger_squares |= Self::attacks_rook(sq, bb_enemies, bb_allies_without_king);
        }
        for sq in board.get_piece_positions(&PieceType::Queen.as_colored_piece(enemy_color)) {
            danger_squares |= Self::attacks_bishop(sq, bb_enemies, bb_allies_without_king);
            danger_squares |= Self::attacks_rook(sq, bb_enemies, bb_allies_without_king);
        }

        danger_squares.into()
    }

    /// Generates the checkers BB for a given square and color. Square is the king square. Checkers are from the opposite color
    fn generate_checkers(king_pos: Square, color: Color, board: &Board) -> BitBoard {
        let attack_color = color.opposite();
        let ally = **board.get_bb_occupied(&color);
        let enemy = **board.get_bb_occupied(&attack_color);
        let enemy_bb = board.get_bb_pieces()[attack_color];

        let mut checkers = 0u64;

        //check for attacks from non-sliding pieces
        //no need to check for king attacks, as a king can't attack another king
        checkers |= MoveGeneration::attacks_knight(king_pos, ally) & *enemy_bb[PieceType::Knight];
        checkers |=
            MoveGeneration::attacks_pawn(king_pos, enemy, ally, color) & *enemy_bb[PieceType::Pawn];

        //if no sliding pieces are available, we won't need to check for attacks
        if *board.bb_sliders[attack_color] == 0 {
            return checkers.into();
        }

        checkers |= MoveGeneration::attacks_bishop(king_pos, enemy, ally)
            & (*enemy_bb[PieceType::Bishop] | *enemy_bb[PieceType::Queen]);
        checkers |= MoveGeneration::attacks_rook(king_pos, enemy, ally)
            & (*enemy_bb[PieceType::Rook] | *enemy_bb[PieceType::Queen]);

        checkers.into()
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

        let queen_side_possible = (queen_side_free & (all | attacked)) == 0;

        if queen_side_possible {
            CastleType::KING_DEST[CastleRights::to_index(&color, &CastleType::QueenSide) as usize]
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

        let king_side_possible = (king_side_free & (all | attacked)) == 0;

        if king_side_possible {
            CastleType::KING_DEST[CastleRights::to_index(&color, &CastleType::KingSide) as usize]
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
        if all.is_occupied(&Square::new(index)) {
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

        let en_passant_mask = en_passant_square.to_mask();
        let sq_mask = sq.to_mask();

        //check for if the en passant capture would expose a discovered attack on the king
        //by removing the pawn and the en passant pawn from the board, we can check if there
        //is a rook or queen attacking the king by using the rook horizontal attack pattern from the king square
        //and check if this ray would attack a enemy rook or queen
        let enemy_without_pawn = enemy & !(en_passant_mask);
        let ally_without_pawn = ally & !(sq_mask);
        let rank_attack_ray =
            attack_pattern::rook_attacks_horizontal(enemy_without_pawn, ally_without_pawn, king_sq);

        if rank_attack_ray & enemy_without_pawn != 0 {
            let horizontal_rook_attack = BitBoard::from(rank_attack_ray & enemy_without_pawn)
                .get_occupied()
                .any(|square| {
                    board
                        .get_sq_piece(&square)
                        .map(|piece| matches!(piece.ptype(), PieceType::Rook | PieceType::Queen))
                        .is_some()
                });

            //if the is a rook or queen attacking the king, the en passant is invalid
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

    /// Generates pseudo-legal moves for a given piece on a given square
    pub fn generate_pseudo_legal_moves(
        square: Square,
        piece: Piece,
        enemy: BitBoard,
        ally: BitBoard,
        board_state: &BoardState,
    ) -> Option<Vec<Move>> {
        let mut moves: Vec<Move> = Vec::new();

        let enemy = *enemy;
        let ally = *ally;

        match piece.ptype() {
            PieceType::Pawn => {
                let attacks = Self::attacks_pawn(square, enemy, ally, piece.color());
                BitBoard::from(attacks).get_occupied().for_each(|sq| {
                    if sq.rank() == piece.color().promotion_rank() {
                        MoveFlag::PAWN_PROMOTION_FLAGS.into_iter().for_each(|flag| {
                            moves.push(Move::new(square, sq, flag));
                        });
                    } else {
                        moves.push(Move::new(square, sq, MoveFlag::Normal));
                    }
                });

                if square.rank() == piece.color().pawn_rank() {
                    let double_push =
                        Self::moves_pawn_double_push(square, enemy, ally, piece.color());
                    BitBoard::from(double_push).get_occupied().for_each(|sq| {
                        moves.push(Move::new(square, sq, MoveFlag::DoublePawnPush));
                    });
                }

                // let en_passant_attack = Self::attacks_pawn_en_passant(
                //     square,
                //     piece.color(),
                //     board_state.en_passant.as_ref(),
                //     enemy,
                //     ally,

                // );
                // BitBoard::from(en_passant_attack)
                //     .get_occupied()
                //     .for_each(|sq| {
                //         moves.push(Move::new(square, sq, MoveFlag::EnPassant));
                //     });

                let pawn_moves = Self::moves_pawn(square, enemy, ally, piece.color());
                BitBoard::from(pawn_moves).get_occupied().for_each(|sq| {
                    if sq.rank() == piece.color().promotion_rank() {
                        MoveFlag::PAWN_PROMOTION_FLAGS.into_iter().for_each(|flag| {
                            moves.push(Move::new(square, sq, flag));
                        });
                    } else {
                        moves.push(Move::new(square, sq, MoveFlag::Normal));
                    }
                });
            }
            PieceType::Knight => {
                let attacks = Self::attacks_knight(square, ally);
                BitBoard::from(attacks).get_occupied().for_each(|sq| {
                    moves.push(Move::new(square, sq, MoveFlag::Normal));
                });
            }
            PieceType::Bishop => {
                let attacks = Self::attacks_bishop(square, enemy, ally);
                BitBoard::from(attacks).get_occupied().for_each(|sq| {
                    moves.push(Move::new(square, sq, MoveFlag::Normal));
                });
            }
            PieceType::Rook => {
                let attacks = Self::attacks_rook(square, enemy, ally);
                BitBoard::from(attacks).get_occupied().for_each(|sq| {
                    moves.push(Move::new(square, sq, MoveFlag::Normal));
                });
            }
            PieceType::Queen => {
                let mut attacks = Self::attacks_rook(square, enemy, ally);
                attacks |= Self::attacks_bishop(square, enemy, ally);
                BitBoard::from(attacks).get_occupied().for_each(|sq| {
                    moves.push(Move::new(square, sq, MoveFlag::Normal));
                });
            }
            PieceType::King => {
                let attacks = Self::attacks_king(square, ally);
                BitBoard::from(attacks).get_occupied().for_each(|sq| {
                    moves.push(Move::new(square, sq, MoveFlag::Normal));
                });

                // let castle_moves = Self::moves_king_castle(square, enemy, ally, 0, piece.color());
                // BitBoard::from(castle_moves).get_occupied().for_each(|sq| {
                //     match sq {
                //         s if s == CastleType::KING_DEST[0] || s == CastleType::KING_DEST[2] => {
                //             moves.push(Move::new(square, sq, MoveFlag::KingSideCastle));
                //         }
                //         s if s == CastleType::KING_DEST[1] || s == CastleType::KING_DEST[3] => {
                //             moves.push(Move::new(square, sq, MoveFlag::QueenSideCastle));
                //         }
                //         _ => (),
                //     };
                // });
            }
        };

        Some(moves)
    }

    /// Generates all legal moves for the current board state
    pub fn generate_all_moves(board: &Board) -> Vec<Move> {
        let bb = board.get_bb_occupied(&board.side_to_move);
        let mut moves = Vec::with_capacity(218); // 218 is the maximum number of moves possible in a position

        bb.get_occupied().for_each(|sq| {
            let piece = board
                .get_sq_piece(&sq)
                .expect("There has to be a piece on this board or the bb are out of sync");
            let enemy = board.get_bb_occupied(&board.side_to_move.opposite());
            let ally = board.get_bb_occupied(&board.side_to_move);

            let sq_moves =
                Self::generate_pseudo_legal_moves(sq, piece, *enemy, *ally, &board.current_state);

            if let Some(sq_moves) = sq_moves {
                moves.extend(sq_moves);
            }
        });

        //TODO filter out illegal moves
        Self::filter_legal_moves(moves, board)
    }

    pub fn filter_legal_moves(moves: Vec<Move>, board: &Board) -> Vec<Move> {
        moves
            .into_iter()
            .filter(|m| {
                let mut board = board.clone();
                let correct = board.make_move(m, false, false);

                if correct.is_err() {
                    return false;
                }
                let in_check = board.in_check(&board.side_to_move.opposite());

                !in_check
            })
            .collect()
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
