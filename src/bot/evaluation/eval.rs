pub type Eval = i32;

pub const NEG_INF: Eval = i32::MIN + 1;
pub const POS_INF: Eval = i32::MAX - 1;

pub const DRAW: Eval = 0;
pub const MATE: Eval = 10000;

pub fn is_mate_score(score: Eval) -> bool {
    if score == NEG_INF || score == POS_INF {
        return false;
    }

    score.abs() >= MATE - u8::MAX as i32
}

pub fn correct_mate_score(eval: Eval, ply_from_root: u8) -> Eval {
    assert!(is_mate_score(eval));

    match eval.signum() {
        1 => MATE - ply_from_root as Eval,
        -1 => -MATE + ply_from_root as Eval,
        _ => unreachable!(),
    }
}
