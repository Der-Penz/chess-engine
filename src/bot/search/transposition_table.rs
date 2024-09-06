use crate::{bot::evaluation::eval::Eval, game::Move};

pub struct TranspositionTableEntry {
    pub zobrist: u64,
    pub depth: u8,
    pub eval: Eval,
    pub node_type: NodeType,
    pub best_move: Option<Move>,
}

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

impl NodeType {
    /// Returns the node type based on the alpha beta window and the original alpha value.
    pub fn type_from_eval(alpha: Eval, original_alpha: Eval, beta: Eval) -> Self {
        if alpha <= original_alpha {
            NodeType::UpperBound
        } else if alpha >= beta {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        }
    }
}

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

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
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
    /// Returns None if the entry was not inserted.
    /// Depending on the management strategy, the entry might not be inserted if the table is full.
    pub fn insert(&mut self, key: u64, entry: TranspositionTableEntry, is_pv: bool) {
        if !self.enabled {
            return;
        }

        let index = self.index(key);
        let existing_entry = self.entries[index].as_ref();

        //pv nodes are always inserted
        if is_pv {
            if existing_entry.is_none() {
                self.entries[index] = Some(entry);
                self.count += 1;
                return;
            }

            self.entries[index] = Some(entry);
        } else {
            if existing_entry.is_none() {
                self.entries[index] = Some(entry);
                self.count += 1;
                return;
            }

            let existing_entry = existing_entry.unwrap();
            if entry.depth >= existing_entry.depth {
                self.entries[index] = Some(entry);
                return;
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
