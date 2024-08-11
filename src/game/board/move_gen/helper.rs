use crate::game::{
    bit_manipulation::{bit_scan_lsb, drop_lsb, north_east, north_west, south_east, south_west},
    board::bit_board::BitBoard,
    Board, Color, PieceType, Square,
};

use super::{
    attack_pattern::{self, direction_mask::CONNECTION_MASK},
    MoveGeneration,
};

/// Data used for move generation.
/// Helper struct to avoid passing multiple arguments to the functions. and to avoid recomputing the same values.
pub struct MoveGenerationData {
    pub color: Color,
    pub color_opp: Color,
    pub ally: u64,
    pub enemy: u64,
    pub occupied: u64,
    pub king_sq: Square,
    pub king_sq_opp: Square,
    pub king_mask: u64,
}

impl MoveGenerationData {
    pub fn from_board(board: &Board) -> Self {
        let color = board.side_to_move;
        let color_opp = color.opposite();
        let ally = *board.bb_occupied[color];
        let enemy = *board.bb_occupied[color_opp];
        let occupied = ally | enemy;
        let king_sq = board.get_king_pos(color);
        let king_sq_opp = board.get_king_pos(color_opp);
        let king_mask = king_sq.to_mask();
        Self {
            color,
            color_opp,
            ally,
            enemy,
            occupied,
            king_sq,
            king_sq_opp,
            king_mask,
        }
    }
}

pub struct MoveGenerationMasks {
    pub king_danger: u64,
    pub checkers: u64,
    pub in_check: bool,
    pub multi_check: bool,
    pub diagonal_pinned: u64,
    pub orthogonal_pinned: u64,
    pub push_mask: u64,
    pub capture_mask: u64,
    pub push_capture_mask: u64,
}

impl std::default::Default for MoveGenerationMasks {
    fn default() -> Self {
        Self {
            king_danger: 0,
            checkers: 0,
            in_check: false,
            multi_check: false,
            diagonal_pinned: 0,
            orthogonal_pinned: 0,
            push_mask: 0,
            capture_mask: 0,
            push_capture_mask: 0,
        }
    }
}

impl MoveGenerationMasks {
    pub fn calculate_king_danger(&mut self, data: &MoveGenerationData, board: &Board) {
        //King attacks
        self.king_danger |= MoveGeneration::attacks_king(data.king_sq_opp, 0);

        //Pawn attacks
        let bb_pawns = *board.get_bb_for(PieceType::Pawn.as_colored_piece(data.color_opp));
        if data.color_opp == Color::White {
            self.king_danger |= north_east(bb_pawns, 1);
            self.king_danger |= north_west(bb_pawns, 1);
        } else {
            self.king_danger |= south_east(bb_pawns, 1);
            self.king_danger |= south_west(bb_pawns, 1);
        }

        //Knight attacks
        for knight_sq in
            board.get_piece_positions(PieceType::Knight.as_colored_piece(data.color_opp))
        {
            self.king_danger |= MoveGeneration::attacks_knight(knight_sq, 0);
        }

        //if we have no sliding pieces, we can skip
        if *board.bb_sliders[data.color_opp] != 0 {
            let occupied_without_king = data.occupied & !data.king_mask;

            //Sliding attacks
            for sq in board.get_piece_positions(PieceType::Bishop.as_colored_piece(data.color_opp))
            {
                self.king_danger |= MoveGeneration::attacks_bishop(sq, occupied_without_king, 0);
            }
            for sq in board.get_piece_positions(PieceType::Rook.as_colored_piece(data.color_opp)) {
                self.king_danger |= MoveGeneration::attacks_rook(sq, occupied_without_king, 0);
            }
            for sq in board.get_piece_positions(PieceType::Queen.as_colored_piece(data.color_opp)) {
                self.king_danger |= MoveGeneration::attacks_bishop(sq, occupied_without_king, 0);
                self.king_danger |= MoveGeneration::attacks_rook(sq, occupied_without_king, 0);
            }
        }

        self.in_check = (self.king_danger & data.king_mask) != 0;

        if !self.in_check {
            self.checkers = 0;
            self.multi_check = false;
            return;
        }

        let ally = data.ally;
        let enemy = data.enemy;
        let enemy_bb = board.get_bb_pieces()[data.color_opp];

        //check for attacks from non-sliding pieces
        //no need to check for king attacks, as a king can't attack another king
        self.checkers |=
            MoveGeneration::attacks_knight(data.king_sq, ally) & *enemy_bb[PieceType::Knight];
        self.checkers |= MoveGeneration::attacks_pawn(data.king_sq, enemy, ally, data.color)
            & *enemy_bb[PieceType::Pawn];

        //if no sliding pieces are available, we won't need to check for attacks
        if *board.bb_sliders[data.color_opp] != 0 {
            self.checkers |= MoveGeneration::attacks_bishop(data.king_sq, enemy, ally)
                & (*enemy_bb[PieceType::Bishop] | *enemy_bb[PieceType::Queen]);
            self.checkers |= MoveGeneration::attacks_rook(data.king_sq, enemy, ally)
                & (*enemy_bb[PieceType::Rook] | *enemy_bb[PieceType::Queen]);
        }

        self.multi_check = self.checkers & (self.checkers.wrapping_sub(1)) != 0;
    }

    pub fn calculate_pins(&mut self, data: &MoveGenerationData, board: &Board) {
        //TODO could be faster if I use a table with the directions and & those to (king in direction) & (slider in opposite direction)
        for sq in board.get_bb_rook_slider(data.color_opp).get_occupied() {
            let slider_mask = sq.to_mask();
            let slider_vertical = attack_pattern::rook_attacks_vertical(0, data.king_mask, sq);
            let slider_horizontal = attack_pattern::rook_attacks_horizontal(0, data.king_mask, sq);
            let king_vertical = attack_pattern::rook_attacks_vertical(0, slider_mask, data.king_sq);
            let king_horizontal =
                attack_pattern::rook_attacks_horizontal(0, slider_mask, data.king_sq);
            let ray = king_vertical & slider_vertical;
            if ray != 0 {
                if (ray & data.ally).count_ones() == 1 && ray & data.enemy == 0 {
                    self.orthogonal_pinned |= ray & data.ally;
                }
            }
            let ray = king_horizontal & slider_horizontal;
            if ray != 0 {
                if (ray & data.ally).count_ones() == 1 && ray & data.enemy == 0 {
                    self.orthogonal_pinned |= ray & data.ally;
                }
            }
        }

        for sq in board.get_bb_bishop_slider(data.color_opp).get_occupied() {
            let slider_mask = sq.to_mask();
            let slider_main = attack_pattern::bishop_attacks_main(0, data.king_mask, sq);
            let slider_anti = attack_pattern::bishop_attacks_anti(0, data.king_mask, sq);
            let king_main = attack_pattern::bishop_attacks_main(0, slider_mask, data.king_sq);
            let king_anti = attack_pattern::bishop_attacks_anti(0, slider_mask, data.king_sq);
            let ray = king_main & slider_main;
            if ray != 0 {
                if (ray & data.ally).count_ones() == 1 && ray & data.enemy == 0 {
                    self.diagonal_pinned |= ray & data.ally;
                }
            }
            let ray = king_anti & slider_anti;
            if ray != 0 {
                if (ray & data.ally).count_ones() == 1 && ray & data.enemy == 0 {
                    self.diagonal_pinned |= ray & data.ally;
                }
            }
        }
    }

    //must be called after calculate_king_danger to have the checkers mask available
    pub fn calculate_push_and_capture(&mut self, data: &MoveGenerationData, board: &Board) {
        //if only one check is present, we can capture the checking piece or block the check
        if self.in_check && !self.multi_check {
            self.capture_mask = self.checkers;

            //if we are in check by a sliding piece, we can block the check
            if (*board.bb_sliders[data.color_opp] & self.checkers) != 0 {
                //calculate the squares between the king and the checker
                self.push_mask =
                    CONNECTION_MASK[bit_scan_lsb(self.checkers) as usize][data.king_sq];
            } else {
                //if we are in check by a non-sliding piece, we can only capture the piece
                self.push_mask = 0;
            }
        } else if self.multi_check {
            self.push_mask = 0;
            self.capture_mask = 0;
        } else {
            self.capture_mask = 0xFFFFFFFFFFFFFFFFu64;
            self.push_mask = 0xFFFFFFFFFFFFFFFFu64;
        }

        self.push_capture_mask = self.push_mask | self.capture_mask;
    }
}
