pub struct SearchDiagnostics {
    pub(crate) node_count: u64,
    pub(crate) node_count_qs: u64,
    pub(crate) cut_offs: u64,
}

impl std::default::Default for SearchDiagnostics {
    fn default() -> Self {
        Self {
            node_count: 0,
            node_count_qs: 0,
            cut_offs: 0,
        }
    }
}

impl std::fmt::Display for SearchDiagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Total: {} || Nodes: {}, Nodes QS: {}, Cut offs: {}",
            self.total_nodes(),
            self.node_count,
            self.node_count_qs,
            self.cut_offs
        )
    }
}

impl SearchDiagnostics {
    pub fn total_nodes(&self) -> u64 {
        self.node_count + self.node_count_qs
    }

    #[inline(always)]
    pub fn inc_node(&mut self) {
        self.node_count += 1;
    }

    #[inline(always)]
    pub fn inc_node_qs(&mut self) {
        self.node_count_qs += 1;
    }

    #[inline(always)]
    pub fn inc_cut_offs(&mut self) {
        self.cut_offs += 1;
    }
}
