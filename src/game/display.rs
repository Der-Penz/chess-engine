
use log::info;

use super::{ Board, Square };

pub fn bb_to_string(f: impl Fn(Square) -> Option<String>) -> String {
    let mut repr = String::new();

    repr.push_str(" ");
    for x in 'A'..'I' {
        repr.push_str(&format!(" {}", x));
    }
    repr.push_str("\n");

    Square::iter_ah_81().for_each(|s| {
        if s.file() == 0 {
            repr.push_str(&format!("{}", s.rank() + 1));
        }

        repr.push_str(&format!(" {}", f(s).unwrap_or(" ".to_string())));

        if s.file() == 7 {
            repr.push_str(&format!("  {}\n", s.rank() + 1));
        }
    });

    repr.push_str(" ");
    for x in 'A'..'I' {
        repr.push_str(&format!(" {}", x));
    }
    repr.push_str("\n");

    return repr;
}

pub fn print_bb_ox(o_bb: u64, x_bb: u64) {
    info!(
        "{}",
        bb_to_string(|s| {
            if s.matches(o_bb) {
                Some("o".to_string())
            } else if s.matches(x_bb) {
                Some("x".to_string())
            } else {
                None
            }
        })
    );
}

pub fn print_bb_oxd(o_bb: u64, x_bb: u64, d_bb: u64) {
    info!(
        "{}",
        bb_to_string(|s| {
            if s.matches(o_bb) {
                Some("o".to_string())
            } else if s.matches(x_bb) && s.matches(d_bb) {
                Some(format!("{}{}", "\u{035C}", "□"))
            } else if s.matches(x_bb) {
                Some("x".to_string())
            } else if s.matches(d_bb) {
                Some("□".to_string())
            } else {
                None
            }
        })
    );
}

/// print the board with the x_bb bit board. Pieces are marked if overlapping with x_bb
pub fn print_bb_bbx(bb: &Board, x_bb: u64) {
    info!(
        "{}",
        bb_to_string(|s| {
            match bb.get_field_piece(s.into()) {
                Some(p) => if s.matches(x_bb) {
                    Some(format!("{}{}", "\u{035C}", p.to_string()))
                } else {
                    Some(p.to_string())
                }
                None => if s.matches(x_bb) { Some("x".to_string()) } else { None }
            }
        })
    );
}
