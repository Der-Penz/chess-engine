pub(crate) fn get_current_millis() -> u128 {
    let now = std::time::SystemTime::now();
    let since_the_epoch = now
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
    since_the_epoch
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// A limit that can be applied to a search.
/// None means no limit. (search will continue until a terminal node is reached)
pub enum Limit {
    Time(u128, u128),
    NodeCount(u64),
    Depth(u8),
    None,
}

impl Limit {
    #[inline(always)]
    /// Check if the limit has been reached.
    pub fn is_terminal(&self, nodes: u64, depth: u8) -> bool {
        match self {
            Limit::Time(start_millis, duration_millis) => {
                let now = get_current_millis();
                now - *start_millis > *duration_millis
            }
            Limit::NodeCount(max_nodes) => nodes >= *max_nodes,
            Limit::Depth(max_depth) => depth >= *max_depth,
            Limit::None => false,
        }
    }
}

type LimitsArray = [Limit; 3];

#[derive(Clone, Debug, PartialEq)]
/// A list of limits that can be applied to a search.  
/// The can be a maximum of 3 limits active at the same time.  
/// No limit can be repeated.
pub struct Limits(LimitsArray);

impl Limits {
    pub fn new() -> Self {
        Limits([Limit::None; 3])
    }

    /// Add a limit to the list of limits.
    /// If the limit type is already present, it will be replaced.
    /// If there is an empty slot, the limit will be added there.
    pub fn add_limit(&mut self, limit: Limit) {
        for i in 0..3 {
            match self.0[i] {
                Limit::None => {
                    self.0[i] = limit;
                    return;
                }
                _ => {}
            }
            if std::mem::discriminant(&self.0[i]) == std::mem::discriminant(&limit) {
                self.0[i] = limit;
                return;
            }
        }
    }

    #[inline(always)]
    /// Check if any of the limits has been reached.
    pub fn is_any_terminal(&self, nodes: u64, depth: u8) -> bool {
        self.0.iter().any(|limit| limit.is_terminal(nodes, depth))
    }
}
