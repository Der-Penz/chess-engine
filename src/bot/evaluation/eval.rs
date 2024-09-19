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
