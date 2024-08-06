use chess_bot::{
    game::{board::move_gen::MoveGeneration, Board, Move},
    init_logging,
};

enum PerftMode {
    NodeCount,
    Divide,
    DivideDetailed,
}

impl From<&str> for PerftMode {
    fn from(s: &str) -> Self {
        match s {
            "nodecount" => Self::NodeCount,
            "divide" => Self::Divide,
            "dividedetailed" => Self::DivideDetailed,
            _ => panic!("Invalid mode"),
        }
    }
}

fn main() {
    init_logging();
    let args: Vec<String> = std::env::args().collect();

    let depth = args
        .get(1)
        .expect("Depth must be provided")
        .parse::<u8>()
        .expect("Depth must be a number");

    let mode = args
        .get(2)
        .map(|s| PerftMode::from(s.as_str()))
        .expect("Mode must be provided (nodecount, divide, dividedetailed)");

    let mut board = if let Some(fen) = args.get(3) {
        Board::from_fen(fen).expect("Invalid FEN")
    } else {
        Board::default()
    };

    //get start time
    let start = std::time::Instant::now();
    let perft_result = perft(depth, &mut board);
    let elapsed = start.elapsed();
    match mode {
        PerftMode::NodeCount => {
            let node_count = perft_result
                .iter()
                .fold(0, |acc, (_, counter)| acc + counter.count);
            println!("Total nodes: {}", node_count);
        }
        PerftMode::Divide => {
            for (mov, counter) in perft_result.iter() {
                println!("{}: {}", mov, counter.count);
            }
        }
        PerftMode::DivideDetailed => {
            for (mov, counter) in perft_result.iter() {
                println!("{}: {}", mov, counter);
            }

            let total = perft_result
                .iter()
                .map(|x| x.1)
                .reduce(|acc, counter| acc + counter);
            if let Some(total) = total {
                println!();
                println!("Sum:  {}", total);
                let node_count = total.count;
                let nodes_per_second = node_count as f64 / elapsed.as_secs_f64();
                println!("Time taken: {:.3?} s", elapsed.as_secs());
                println!("Nodes per second: {:.*} N/s", 0, nodes_per_second);
            }
        }
    }
}

fn perft(depth: u8, board: &mut Board) -> Vec<(Move, MoveCounter)> {
    if depth == 0 {
        return vec![];
    }
    let move_list = MoveGeneration::generate_legal_moves(board);
    let mut results = Vec::with_capacity(move_list.len());

    for mov in move_list.iter() {
        board
            .make_move(mov, false, false)
            .expect("Move generation should generate only legal moves");
        let mut counter = MoveCounter::default();
        counter += perft_depth(depth - 1, board);
        counter.add_details(board, mov);
        board
            .undo_move(mov, false)
            .expect("Undo move should be valid");
        results.push((*mov, counter));
    }

    results
}

fn perft_depth(depth: u8, board: &mut Board) -> MoveCounter {
    if depth == 0 {
        return MoveCounter::new_one();
    }

    let move_list = MoveGeneration::generate_legal_moves(board);
    let mut counter = MoveCounter::default();
    for mov in move_list.iter() {
        board
            .make_move(mov, false, false)
            .expect("Move generation should generate only legal moves");
        counter += perft_depth(depth - 1, board);
        counter.add_details(board, mov);
        board
            .undo_move(mov, false)
            .expect("Undo move should be valid");
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

impl MoveCounter {
    fn new_one() -> Self {
        Self {
            count: 1,
            captures: 0,
            en_passant: 0,
            castles: 0,
            promotions: 0,
            checks: 0,
        }
    }

    fn add_details(&mut self, board: &Board, mov: &Move) {
        if mov.flag().is_castle() {
            self.castles += 1;
        }
        if mov.flag().is_en_passant() {
            self.en_passant += 1;
        }
        if mov.flag().is_promotion() {
            self.promotions += 1;
        }
        if board.cur_state().captured_piece.is_some() {
            self.captures += 1;
        }
    }
}
