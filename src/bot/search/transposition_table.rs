use crate::{bot::evaluation::eval::Eval, game::Move};

pub struct TranspositionTableEntry {
    pub zobrist: u64,
    pub depth: u8,
    pub eval: Eval,
    pub node_type: NodeType,
    pub best_move: Option<Move>,
}

pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}

impl NodeType {
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

pub enum ReplacementStrategy {
    ReplaceAlways,
    KeepAlways,
    ReplaceOnFull(bool),
    DepthPriority,
}

pub struct TranspositionTable {
    entries: Vec<Option<TranspositionTableEntry>>,
    count: usize,
    max_count: usize,
    replacement_strategy: ReplacementStrategy,
}

impl TranspositionTable {
    pub fn new(mb: f64, replacement_strategy: ReplacementStrategy) -> Self {
        let max_count = TranspositionTable::get_size_from_mb(mb);
        let mut entries = Vec::with_capacity(max_count);
        entries.resize_with(max_count, || None);

        Self {
            entries,
            count: 0,
            max_count,
            replacement_strategy,
        }
    }

    #[inline(always)]
    //linear probing
    fn index(&self, key: u64) -> usize {
        key as usize % self.max_count
    }

    pub fn get_usage(&self) -> f64 {
        self.count as f64 / self.max_count as f64
    }

    pub fn get_entry(&self, key: u64) -> Option<&TranspositionTableEntry> {
        self.entries[self.index(key)].as_ref()
    }

    /// Inserts a new entry into the transposition table.
    /// Returns None if the entry was not inserted.
    /// Depending on the management strategy, the entry might not be inserted if the table is full.
    pub fn insert(&mut self, key: u64, entry: TranspositionTableEntry) -> Option<()> {
        let index = self.index(key);
        match self.replacement_strategy {
            ReplacementStrategy::KeepAlways => {
                if self.count >= self.max_count {
                    return None;
                }

                if self.entries[index].is_some() {
                    return Some(());
                }
                self.entries[index] = Some(entry);
                self.count += 1;
                return Some(());
            }
            ReplacementStrategy::ReplaceAlways => {
                if self.entries[index].is_none() {
                    self.count += 1;
                }
                self.entries[index] = Some(entry);
                return Some(());
            }
            ReplacementStrategy::ReplaceOnFull(should_replace) => {
                if should_replace {
                    self.entries[index] = Some(entry);
                    return Some(());
                }

                if self.count >= self.max_count {
                    info!("Transposition table is full, switch to replacement strategy");
                    self.replacement_strategy = ReplacementStrategy::ReplaceOnFull(true);
                    self.entries[index] = Some(entry);
                    return Some(());
                }

                if self.entries[index].is_some() {
                    return None;
                }

                self.entries[index] = Some(entry);
                self.count += 1;
                return Some(());
            }
            ReplacementStrategy::DepthPriority => {
                if self.entries[index].is_none() {
                    self.entries[index] = Some(entry);
                    self.count += 1;
                    return Some(());
                }

                let existing_entry = self.entries[index].as_mut().unwrap();
                if entry.depth >= existing_entry.depth {
                    *existing_entry = entry;
                    return Some(());
                } else {
                    return None;
                }
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
        self.max_count = Self::get_size_from_mb(mb);
        self.entries.resize_with(self.max_count, || None);

        self.clear();
    }

    pub fn set_replacement_strategy(&mut self, strategy: ReplacementStrategy) {
        self.replacement_strategy = strategy;
    }

    fn get_size_from_mb(mb: f64) -> usize {
        let bytes = mb * 1024_f64 * 1024_f64;
        let entries = bytes / std::mem::size_of::<TranspositionTableEntry>() as f64;
        entries.floor() as usize
    }
}
