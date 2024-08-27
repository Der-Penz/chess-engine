pub mod bot;
pub mod game;
pub mod uci;

use game::{
    board::move_gen::{magic::lazy_static_attack_table_init, MoveGeneration},
    Board, Move,
};
#[cfg(feature = "log_to_file")]
use log::info;
use log::LevelFilter;
use std::{
    io::Write,
    sync::{Arc, Mutex},
    thread,
};
use threadpool::ThreadPool;

pub fn init() {
    lazy_static_attack_table_init();

    #[cfg(not(feature = "log_to_file"))]
    {
        env_logger::builder()
            .format(|buf, record| writeln!(buf, "{}", record.args()))
            .filter(None, LevelFilter::Info)
            .init();
    }

    #[cfg(feature = "log_to_file")]
    {
        let log_file_path = std::env::var("LOG_FILE").unwrap_or("logs.log".to_string());
        let log_file = std::fs::File::create(log_file_path).unwrap();
        env_logger::Builder::from_default_env()
            .format(|buf, record| writeln!(buf, "[{}] - {}", record.level(), record.args()))
            .target(env_logger::Target::Pipe(Box::new(log_file)))
            .filter(None, LevelFilter::Info)
            .init();
        info!("Log to file enabled");
    }
}

pub fn perft(depth: u8, board: &mut Board, multithreaded: bool) -> Vec<(Move, MoveCounter)> {
    if depth == 0 {
        return vec![];
    }
    let mut move_gen = MoveGeneration::new();
    let move_list = move_gen.generate_legal_moves(board);
    let results = Arc::new(Mutex::new(Vec::with_capacity(move_list.len())));
    let pool = ThreadPool::new(thread::available_parallelism().unwrap().into());
    for mov in move_list.clone().iter() {
        let mov = mov.clone();
        if multithreaded {
            let mut board_clone = board.clone();
            let results = Arc::clone(&results);
            pool.execute(move || {
                let mut counter = MoveCounter::default();
                board_clone
                    .make_move(&mov, false, false)
                    .expect("Move generation should generate only legal moves");
                if depth == 1 {
                    counter.add_details(&board_clone, &mov);
                }
                counter += perft_depth(depth - 1, &mut board_clone);
                board_clone
                    .undo_move(&mov, false)
                    .expect("Undo move should be valid");
                results.lock().unwrap().push((mov, counter));
            });
        } else {
            let mut counter = MoveCounter::default();
            board
                .make_move(&mov, false, false)
                .expect("Move generation should generate only legal moves");
            if depth == 1 {
                counter.add_details(board, &mov);
            }
            counter += perft_depth(depth - 1, board);
            board
                .undo_move(&mov, false)
                .expect("Undo move should be valid");
            results.lock().unwrap().push((mov, counter));
        }
    }

    if multithreaded {
        pool.join();
    }

    let x = results.lock().unwrap().clone();
    x
}

fn perft_depth(depth: u8, board: &mut Board) -> MoveCounter {
    if depth == 0 {
        return MoveCounter::new_one();
    }

    let mut move_gen = MoveGeneration::new();
    let move_list = move_gen.generate_legal_moves(board);
    let mut counter = MoveCounter::default();
    for mov in move_list.iter() {
        board
            .make_move(mov, false, false)
            .expect("Move generation should generate only legal moves");
        if depth == 1 {
            counter.add_details(board, mov);
        }
        counter += perft_depth(depth - 1, board);
        board
            .undo_move(mov, false)
            .expect("Undo move should be valid");
    }
    counter
}

#[derive(Debug, Clone, Copy)]
pub struct MoveCounter {
    pub count: u64,
    pub captures: u64,
    pub en_passant: u64,
    pub castles: u64,
    pub promotions: u64,
    pub checks: u64,
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
        if board.in_check() {
            self.checks += 1;
        }
    }
}
