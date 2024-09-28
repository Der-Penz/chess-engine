use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

pub type Eval = i32;

pub const NEG_INF: Eval = i32::MIN + 1;
pub const POS_INF: Eval = i32::MAX - 1;

pub const DRAW: Eval = 0;
pub const MATE: Eval = 100000;

/// Check if the evaluation score is a mate score.
/// Mate scores are defined as a score that is greater than or equal to MATE - 255 or less than or equal to -MATE + 255.
pub fn is_mate_score(score: Eval) -> bool {
    if score == NEG_INF || score == POS_INF {
        return false;
    }

    score.abs() >= MATE - u8::MAX as i32
}

/// Display the evaluation score in a human-readable format.
/// Converts mate scores to "mate X" where X is the number of moves to mate.
/// Converts centipawn scores to "cp X" where X is the centipawn (divided by 100) value.
pub fn display_eval(eval: Eval) -> String {
    if is_mate_score(eval) {
        let mate_in = (MATE - eval.abs()) as f32 / 2.0;
        if eval > 0 {
            format!("mate {}", mate_in.ceil() as i32)
        } else {
            format!("mate -{}", mate_in.ceil() as i32)
        }
    } else {
        format!("cp {}", eval as f64 / 100.0)
    }
}

#[derive(Serialize, Deserialize)]
pub struct EvalScores {
    pub endgame_material_start: Eval,
    pub piece_values: [Eval; 6],
    pub king_pawn_shield: Eval,
    pub king_not_castled: Eval,
    pub bishop_pair: Eval,
    pub isolated_pawn: Eval,
    pub doubled_pawn: Eval,
    pub rook_semi_open_file: Eval,
    pub rook_open_file: Eval,
    pub rook_on_closed: Eval,
    pub rook_on_seventh: Eval,
    pub rook_connected: Eval,
}

impl EvalScores {
    pub fn load_from_file(path: &str) -> Result<Self, ()> {
        let content = std::fs::read_to_string(path).map_err(|_| ())?;
        let scores: EvalScores = serde_json::from_str(&content).map_err(|_| ())?;
        Ok(scores)
    }
}

fn read_start_eval_score() -> EvalScores {
    let file_path = std::env::var("EVAL_SCORES").unwrap_or_else(|_| "eval_scores.json".to_string());
    info!("Loading eval scores from file: {}", file_path);
    //log all env vars
    for (key, value) in std::env::vars() {
        info!(
            "
        key: {} value: {}",
            key, value
        );
    }
    EvalScores::load_from_file(&file_path).expect("Missing eval scores file")
}

lazy_static! {
    pub static ref EVAL_SCORES: EvalScores = read_start_eval_score();
}
