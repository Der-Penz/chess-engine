use crate::game::{evaluation, Board, Color, MoveError};

pub fn min_max(board: &mut Board, depth: u8, color: Color) -> Result<i32, MoveError> {
    if depth == 0 {
        return Ok(evaluation::evaluate_board(board));
    }

    let mut best_score = match color {
        Color::WHITE => i32::MIN,
        Color::BLACK => i32::MAX,
    };

    for mv in board.get_all_possible_moves() {
        let undo = board.play_move(&mv, false)?;
        let score = min_max(board, depth - 1, color.opposite())?;
        board.undo_move(&undo);
        best_score = match color {
            Color::WHITE => best_score.max(score),
            Color::BLACK => best_score.min(score),
        };
    }

    Ok(best_score)
}
