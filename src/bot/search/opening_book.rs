use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

use rand::Rng;

use crate::game::{Board, Move};

pub struct BookMove {
    pub uci_move: String,
    pub num_plays: u32,
}

pub struct OpeningBook {
    moves: HashMap<String, (Vec<BookMove>, u32)>,
    enabled: bool,
}

/// #### Opening book implementation
/// The opening book is a simple text file that contains a list of positions and the most played moves in those positions.
/// Can be used to play the opening phase of the game.
/// If disabled, will return None for all queries.
impl OpeningBook {
    /// Creates a new opening book from a file  
    /// The file must be in the following format:  
    /// *pos \<fen without half move and ply clock>  
    /// \<move> \<number of plays>+  
    /// pos \<fen without half move and ply clock>  
    /// ...*  
    pub fn new(book_file: String) -> Result<Self, ()> {
        let file = File::open(book_file).map_err(|_| ())?;

        let mut moves = HashMap::new();
        let mut reader = BufReader::new(file);
        let mut buf = String::new();

        let mut cur_pos = String::new();
        let mut cur_moves = Vec::new();
        let mut plays_counter = 0;
        loop {
            buf.clear();
            let res = reader.read_line(&mut buf).map_err(|_| ())?;

            if res == 0 {
                break;
            }

            if buf.starts_with("pos ") {
                if cur_pos != "" {
                    moves.insert(cur_pos, (cur_moves, plays_counter));
                }

                cur_pos = buf.trim_start_matches("pos ").trim().to_string();
                cur_moves = Vec::new();
                plays_counter = 0;
                continue;
            }

            if let Some(book_move) = OpeningBook::parse_mov(buf.trim().to_string()) {
                plays_counter += book_move.num_plays;
                cur_moves.push(book_move);
            }
        }

        Ok(Self {
            moves,
            enabled: true,
        })
    }

    fn parse_mov(pos: String) -> Option<BookMove> {
        let (mv, num_plays) = pos.split_once(" ")?;

        let num_plays = num_plays.parse::<u32>().ok()?;
        Some(BookMove {
            uci_move: mv.trim().to_string(),
            num_plays,
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_moves_entry(&self, board: &Board) -> Option<&(Vec<BookMove>, u32)> {
        if !self.enabled {
            return None;
        }

        let fen = board.to_fen();

        let fen = fen.split(" ").take(4).collect::<Vec<&str>>().join(" ");

        self.moves.get(&fen)
    }

    /// Returns the most played move in the opening book for the current position
    pub fn get_most_played_book_move(&self, board: &Board) -> Option<Move> {
        let (moves, _) = self.get_moves_entry(board)?;

        let mut max = 0;
        let mut best_move = None;
        for mov in moves {
            if mov.num_plays > max {
                max = mov.num_plays;
                best_move = Some(mov);
            }
        }

        match best_move {
            Some(mv) => Move::from_uci_notation(&mv.uci_move, board).ok(),
            None => todo!(),
        }
    }

    /// Returns a random move from the opening book for the current position weighted by the number of plays
    pub fn get_random_book_move(&self, board: &Board) -> Option<Move> {
        let (moves, num_plays) = self.get_moves_entry(board)?;

        let mut rng = rand::thread_rng();
        let rand_num = rng.gen_range(0..*num_plays);

        let mut count = 0;
        for mov in moves {
            count += mov.num_plays;
            if rand_num <= count {
                return Move::from_uci_notation(&mov.uci_move, board).ok();
            }
        }
        None
    }
}
