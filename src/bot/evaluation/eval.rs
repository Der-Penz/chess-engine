pub type Eval = i32;

pub const NEG_INF: Eval = i32::MIN + 1;
pub const POS_INF: Eval = i32::MAX - 1;

pub const DRAW: Eval = 0;
pub const MATE: Eval = 100000;

pub fn is_mate_score(score: Eval) -> bool {
    if score == NEG_INF || score == POS_INF {
        return false;
    }

    score.abs() >= MATE - u8::MAX as i32
}

pub fn ply_from_mate(score: Eval) -> u8 {
    (MATE - score.abs()) as u8
}
