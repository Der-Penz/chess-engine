use super::{move_gen::MoveGeneration, Board};
use crate::game::{Color, PieceType};

impl GameResult {
    pub fn get_game_result(board: &Board, move_gen: Option<MoveGeneration>) -> GameResult {
        let mut move_gen = match move_gen {
            Some(mg) => mg,
            None => MoveGeneration::new(),
        };

        let moves = if let Some(moves) = move_gen.get_computed_moves() {
            moves
        } else {
            move_gen.generate_legal_moves(board);
            move_gen
                .get_computed_moves()
                .expect("Must have moves after generating them")
        };

        if moves.is_empty() {
            let masks = move_gen
                .get_move_masks()
                .expect("Must have move masks if moves have been generated");
            if masks.in_check {
                return GameResult::Mate(board.side_to_move);
            } else {
                return GameResult::Stalemate;
            }
        }

        if board.current_state.ply_clock >= 100 {
            return GameResult::FiftyMoveRule;
        }

        //check for threefold repetition
        let rep_count = board.repetition_history.iter().fold(0, |count, x| {
            if *x == board.current_state.zobrist {
                count + 1
            } else {
                count
            }
        });
        if rep_count >= 3 {
            return GameResult::Repetition;
        }

        if Self::check_insufficient_material(&board) {
            return GameResult::InsufficientMaterial;
        }

        GameResult::InProgress
    }

    fn check_insufficient_material(board: &Board) -> bool {
        //if there are any pawns, rooks, or queens, the game is not in an insufficient material state
        if *board.bb_pieces[Color::White][PieceType::Pawn] > 0
            || *board.bb_pieces[Color::Black][PieceType::Pawn] > 0
            || *board.get_bb_rook_slider(Color::White) > 0
            || *board.get_bb_rook_slider(Color::Black) > 0
        {
            return false;
        }

        let num_white_bishops = board.bb_pieces[Color::White][PieceType::Bishop].count_ones();
        let num_black_bishops = board.bb_pieces[Color::Black][PieceType::Bishop].count_ones();
        let num_white_knights = board.bb_pieces[Color::White][PieceType::Knight].count_ones();
        let num_black_knights = board.bb_pieces[Color::Black][PieceType::Knight].count_ones();
        let num_white_minors = num_white_bishops + num_white_knights;
        let num_black_minors = num_black_bishops + num_black_knights;
        let num_minors = num_white_minors + num_black_minors;

        //lone king or king vs king with minor piece is insufficient material
        if num_minors <= 1 {
            return true;
        }

        if num_minors == 2 && num_white_bishops == 1 && num_black_bishops == 1 {
            let is_white_bishop_light =
                board.bb_pieces[Color::White][PieceType::Bishop].trailing_zeros() % 2 == 0;
            let is_black_bishop_light =
                board.bb_pieces[Color::Black][PieceType::Bishop].trailing_zeros() % 2 == 0;
            return is_white_bishop_light == is_black_bishop_light;
        }

        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    InProgress,
    Mate(Color),
    Stalemate,
    Repetition,
    FiftyMoveRule,
    InsufficientMaterial,
    Draw,
    Resign(Color), // can be used to indicate a player has resigned, not returned by get_game_result
    Timeout(Color), // can be used to indicate a player has run out of time, not returned by get_game_result
}

impl GameResult {
    pub fn is_game_over(&self) -> bool {
        !matches!(self, GameResult::InProgress)
    }

    pub fn is_draw(&self) -> bool {
        matches!(
            self,
            GameResult::Draw
                | GameResult::Stalemate
                | GameResult::Repetition
                | GameResult::FiftyMoveRule
                | GameResult::InsufficientMaterial
        )
    }
}

impl From<GameResult> for &str {
    fn from(value: GameResult) -> Self {
        match value {
            GameResult::InProgress => "*",
            GameResult::Mate(color) => match color {
                Color::White => "0-1",
                Color::Black => "1-0",
            },
            GameResult::Stalemate => "1/2-1/2",
            GameResult::Repetition => "1/2-1/2",
            GameResult::FiftyMoveRule => "1/2-1/2",
            GameResult::InsufficientMaterial => "1/2-1/2",
            GameResult::Draw => "1/2-1/2",
            GameResult::Resign(color) => match color {
                Color::White => "0-1",
                Color::Black => "1-0",
            },
            GameResult::Timeout(color) => match color {
                Color::White => "0-1",
                Color::Black => "1-0",
            },
        }
    }
}

mod test {

    #[test]
    fn test_stalemate() {
        use crate::game::{Board, GameResult};

        let stalemate_fens = vec![
            "4k3/4P3/4K3/8/8/8/8/8 b - - 0 1",
            "7k/8/6Q1/8/8/8/8/K7 b - - 0 1",
            "kb5R/8/K7/6P1/8/8/8/8 b - - 0 1",
            "7R/8/5K2/6P1/8/1Q6/p7/k7 b - - 0 1",
            "4kBRK/4P1PP/8/8/8/8/8/8 w - - 0 1",
            "8/p4KBk/P7/8/8/8/8/8 b - - 0 1",
        ];

        for fen in stalemate_fens {
            let board = Board::from_fen(fen).unwrap();
            let res = GameResult::get_game_result(&board, None);
            assert_eq!(res, GameResult::Stalemate);
        }
    }

    #[test]
    fn test_mate() {
        use crate::game::Color;
        use crate::game::{Board, GameResult};

        let mate_fens = vec![
            (
                "r1b1q1n1/1p1k1Npp/3Q4/2p5/P7/8/3PBPPP/nNBK3R b - - 0 17",
                Color::Black,
            ),
            ("6k1/1p2np1p/p1p2qr1/3pK3/8/8/8/8 w - - 4 55", Color::White),
            ("q3q3/4Bk2/Kp6/p6p/2n3p1/8/8/8 w - - 4 48", Color::White),
            ("2q3K1/4r3/1k6/p7/8/3p4/8/8 w - - 7 55", Color::White),
        ];

        for (fen, mated_color) in mate_fens {
            let board = Board::from_fen(fen).unwrap();
            let res = GameResult::get_game_result(&board, None);
            assert_eq!(res, GameResult::Mate(mated_color));
        }
    }
}
