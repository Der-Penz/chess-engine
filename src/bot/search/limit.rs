#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Limit {
    Time(u128, u128),
    NodeCount(u64),
    Depth(u8),
    None,
}

impl Limit {
    pub fn is_terminal(&self, nodes: u64, depth: u8) -> bool {
        match self {
            Limit::Time(start_millis, duration_millis) => {
                let now = std::time::SystemTime::now();
                let since_the_epoch = now
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis();
                since_the_epoch - *start_millis > *duration_millis
            }
            Limit::NodeCount(max_nodes) => nodes >= *max_nodes,
            Limit::Depth(max_depth) => depth > *max_depth,
            Limit::None => false,
        }
    }
}

type LimitsArray = [Limit; 3];

#[derive(Clone, Debug, PartialEq)]
pub struct Limits(LimitsArray);

impl Limits {
    pub fn new() -> Self {
        Limits([Limit::None; 3])
    }

    pub fn add_limit(&mut self, limit: Limit) {
        for i in 0..3 {
            if self.0[i] == Limit::None {
                self.0[i] = limit;
                return;
            }
        }
    }

    #[inline(always)]
    pub fn is_any_terminal(&self, nodes: u64, depth: u8) -> bool {
        self.0.iter().any(|limit| limit.is_terminal(nodes, depth))
    }
}
