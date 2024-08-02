use std::vec;

use chess_bot::game::{move_gen::MoveGeneration, Board, Move};

fn main() {
    //read args depth and fen from command line
    //validate if depth must be provided fen is optional and default is startpos
    let args: Vec<String> = std::env::args().collect();

    let depth = args
        .get(1)
        .expect("Depth must be provided")
        .parse::<u8>()
        .expect("Depth must be a number");

    let mut board = if let Some(fen) = args.get(2) {
        println!("FEN provided: {}", fen);
        Board::from_fen(fen).expect("Invalid FEN")
    } else {
        Board::default()
    };

    println!(
        "Perft result with depth {} for FEN: {}",
        depth,
        board.to_fen()
    );

    let res = perft(depth, &mut board, false);

    for (mov, counter) in res.iter() {
        let mut mov_str = String::with_capacity(30);
        mov_str.push_str(format!("{}", mov).as_str());
        if mov_str.len() < 30 {
            mov_str.push_str(&" ".repeat(30 - mov_str.len()));
        }
        println!("{} - {}", mov_str, counter.count);
        // println!("{} - {}", mov_str, counter);
    }
    println!(
        "Total nodes: {}",
        res.iter().map(|(_, c)| c.count).sum::<usize>()
    );

    // for depth in 0..=depth {
    //     let moves_at_depth = perft_depth(depth, &mut board);
    //     println!("Depth: {} - {}", depth, moves_at_depth);
    // }
}

fn perft(depth: u8, board: &mut Board, simple: bool) -> Vec<(Move, MoveCounter)> {
    if depth == 0 {
        return vec![];
    }
    let mut results = Vec::new();
    let moves = MoveGeneration::generate_all_moves(&board);
    for idx in 0..moves.len() {
        let mut counter = MoveCounter::default();
        let mov = moves[idx];
        board.make_move(&mov, false, false).expect("Valid move");
        counter += perft_depth(depth - 1, board);
        match mov.flag() {
            f if f.is_en_passant() => counter.en_passant += 1,
            f if f.is_castle() => counter.castles += 1,
            f if f.is_promotion() => counter.promotions += 1,
            _ => {}
        }
        if board.cur_state().captured_piece.is_some() {
            counter.captures += 1;
        }
        if board.in_check(&board.side_to_move()) {
            counter.checks += 1;
        }
        board.undo_move(&mov, false).expect("Valid undo move");
        results.push((mov, counter));
    }
    results
}

fn perft_depth(depth: u8, board: &mut Board) -> MoveCounter {
    if depth == 0 {
        return MoveCounter {
            count: 1,
            ..Default::default()
        };
    }
    let mut counter = MoveCounter::default();

    let moves = MoveGeneration::generate_all_moves(&board);
    for mov in moves {
        board.make_move(&mov, false, false).expect("Valid move");
        counter += perft_depth(depth - 1, board);
        match mov.flag() {
            f if f.is_en_passant() => counter.en_passant += 1,
            f if f.is_castle() => counter.castles += 1,
            f if f.is_promotion() => counter.promotions += 1,
            _ => {}
        }
        if board.cur_state().captured_piece.is_some() {
            counter.captures += 1;
        }
        if board.in_check(&board.side_to_move()) {
            counter.checks += 1;
        }
        board.undo_move(&mov, false).expect("Valid undo move");
    }
    counter
}

#[derive(Debug, Clone, Copy)]
struct MoveCounter {
    count: usize,
    captures: usize,
    en_passant: usize,
    castles: usize,
    promotions: usize,
    checks: usize,
}

impl std::ops::Add for MoveCounter {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            count: self.count + other.count,
            captures: self.captures + other.captures,
            en_passant: self.en_passant + other.en_passant,
            castles: self.castles + other.castles,
            promotions: self.promotions + other.promotions,
            checks: self.checks + other.checks,
        }
    }
}

impl std::ops::AddAssign for MoveCounter {
    fn add_assign(&mut self, other: Self) {
        self.count += other.count;
        self.captures += other.captures;
        self.en_passant += other.en_passant;
        self.castles += other.castles;
        self.promotions += other.promotions;
        self.checks += other.checks;
    }
}

impl std::default::Default for MoveCounter {
    fn default() -> Self {
        Self {
            count: 0,
            captures: 0,
            en_passant: 0,
            castles: 0,
            promotions: 0,
            checks: 0,
        }
    }
}

impl std::fmt::Display for MoveCounter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Nodes: {:>10}| Captures: {:>10}| En passant: {:>10}| Castles: {:>10}| Promotions: {:>10} | Checks: {:>10}",
            self.count, self.captures, self.en_passant, self.castles, self.promotions, self.checks
        )
    }
}
