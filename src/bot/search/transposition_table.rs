use crate::{
    bot::evaluation::eval::{is_mate_score, Eval, MATE},
    game::Move,
};

pub struct TranspositionTableEntry {
    pub zobrist: u64,
    pub depth: u8,
    pub eval: Eval,
    pub node_type: NodeType,
    pub best_move: Option<Move>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NodeType {
    /// Exact score of the position was found for the given depth (PV node)
    Exact,
    /// Alpha is the lower bound of the position (Cut Node)
    LowerBound,
    /// Beta is the upper bound of the position (All Node)
    UpperBound,
}

pub const DEFAULT_HASH_SIZE: f64 = 1024_f64;
pub const MAX_HASH_SIZE: f64 = 1024000_f64;
pub const MIN_HASH_SIZE: f64 = 1_f64;

/// #### Transposition Table
/// Cache for storing the results of previous searches  
/// Indexed by the Zobrist hash of the position  
/// Table stores eval and move information for a given position
pub struct TranspositionTable {
    entries: Vec<Option<TranspositionTableEntry>>,
    count: usize,
    max_count: usize,
    enabled: bool,
}

impl TranspositionTable {
    /// Creates a new transposition table with the given size in MB
    pub fn new(mb: f64) -> Self {
        let max_count = TranspositionTable::get_size_from_mb(mb);
        let mut entries = Vec::with_capacity(max_count);
        entries.resize_with(max_count, || None);

        Self {
            entries,
            count: 0,
            max_count,
            enabled: true,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    #[inline(always)]
    //linear probing
    fn index(&self, key: u64) -> usize {
        key as usize % self.max_count
    }

    /// Usage of occupied entries in the transposition table in percentage
    pub fn get_usage(&self) -> f64 {
        self.count as f64 / self.max_count as f64
    }

    ///returns the entry without any checks for depth or hash match
    pub fn get_entry_raw(&self, key: u64) -> Option<&TranspositionTableEntry> {
        if !self.enabled {
            return None;
        }

        self.entries[self.index(key)].as_ref()
    }

    /// Returns the entry if the key matches and the depth is greater or equal to the requested depth
    pub fn get_entry(&self, key: u64, depth: u8) -> Option<&TranspositionTableEntry> {
        if !self.enabled {
            return None;
        }

        let entry = self.entries[self.index(key)].as_ref()?;
        if entry.zobrist == key && entry.depth >= depth {
            Some(entry)
        } else {
            None
        }
    }

    /// Inserts a new entry into the transposition table.  
    /// PV nodes are always inserted even if a PV node was already stored at that index.  
    /// other nodes are inserted if the depth is greater than the existing entry.
    pub fn insert(
        &mut self,
        key: u64,
        depth: u8,
        ply_from_root: u8,
        mut eval: Eval,
        node_type: NodeType,
        best_move: Option<Move>,
    ) {
        if !self.enabled {
            return;
        }

        //correct mate score
        //see more about correcting mate scores:
        //https://github.com/maksimKorzh/chess_programming/blob/9f2dbc2c1bb1f5e405aa9c88cac18840829a29eb/src/bbc/tt_search_mating_scores/TT_mate_scoring.txt
        if is_mate_score(eval) {
            let n = MATE - eval.abs();
            let k = ply_from_root as Eval;
            let distance = n - k;
            eval = (MATE - distance) * eval.signum();
        }

        let entry = TranspositionTableEntry {
            zobrist: key,
            depth,
            node_type,
            best_move,
            eval,
        };

        let index = self.index(key);
        let existing_entry = self.entries[index].as_ref();

        //pv nodes are always inserted
        if entry.node_type == NodeType::Exact {
            if existing_entry.is_none() {
                self.count += 1;
            }
            self.entries[index] = Some(entry);
        } else {
            if let Some(existing_entry) = existing_entry {
                if entry.depth >= existing_entry.depth {
                    self.entries[index] = Some(entry);
                }
            } else {
                self.entries[index] = Some(entry);
                self.count += 1;
            }
        }
    }

    /// Clears the transposition table.
    pub fn clear(&mut self) {
        for entry in self.entries.iter_mut() {
            *entry = None;
        }
        self.count = 0;
    }

    /// Set the size of the transposition table in MB and clears it.
    pub fn set_size(&mut self, mb: f64) {
        self.max_count = TranspositionTable::get_size_from_mb(mb);
        self.entries.resize_with(self.max_count, || None);

        self.clear();
    }

    #[inline(always)]
    /// Corrects a mate score retrieved from the transposition table to the correct mate score distance from the root.  
    /// Mate scores are stores as MATE - N + K (N = distance to mate, K = ply from root) to be relative to the node.  
    /// E.g if a mate in 5 at node Q is found at depth 3, the score of Q is stored as MATE - 5 + 3 = MATE - 2  
    /// to retrieve the correct mate from another node P at depth 2, which has the score of node Q stored, we need subtract K to get MATE - N  
    /// see more about correcting mate scores:
    /// https://github.com/maksimKorzh/chess_programming/blob/9f2dbc2c1bb1f5e405aa9c88cac18840829a29eb/src/bbc/tt_search_mating_scores/TT_mate_scoring.txt
    pub fn correct_retrieved_mate_score(eval: &mut Eval, ply_from_root: u8) {
        if is_mate_score(*eval) {
            *eval = (eval.abs() - ply_from_root as Eval) * eval.signum();
        }
    }

    fn get_size_from_mb(mb: f64) -> usize {
        let mb = if mb < MIN_HASH_SIZE {
            MIN_HASH_SIZE
        } else if mb > MAX_HASH_SIZE {
            MAX_HASH_SIZE
        } else {
            mb
        };

        let bytes = mb * 1024_f64 * 1024_f64;
        let entries = bytes / std::mem::size_of::<TranspositionTableEntry>() as f64;
        entries.floor() as usize
    }
}
