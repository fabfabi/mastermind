/// enum to calculate the number of entries for a strategy
#[derive(Clone, PartialEq, Debug, Copy)]
pub enum StrategyCounter {
    PENDING { count: u16 },            // counting not yet done
    PARTIALLY_FINISHED { count: u16 }, // first count available
    FINISHED { count: u16 },           // this strategy path finished already
                                       //Obsolete,                         // This path has more moves than a known path
                                       //End,                              // this is the last node of the strategy
}
impl StrategyCounter {
    /// return the best case how this could be solved
    ///
    /// Assuming the first shot would create a group for each candidate (incl one that is finished)
    /// and another shot for clearing all the unfinished ones
    pub fn new(n_candidates: usize) -> Self {
        //
        if n_candidates == 1 {
            return Self::FINISHED { count: 1 };
        }
        return Self::PENDING {
            count: 2 * n_candidates as u16 - 1,
        };
    }

    /// returns the count only for those that are (partially )
    pub fn get_count_finished(&self) -> Option<u16> {
        return match self {
            Self::PARTIALLY_FINISHED { count } => Some(*count),
            Self::FINISHED { count } => Some(*count),
            _ => None,
        };
    }

    /// returns the count estimate
    pub fn get_count_estimate(&self) -> u16 {
        return match *self {
            Self::PARTIALLY_FINISHED { count } => count,
            Self::FINISHED { count } => count,
            Self::PENDING { count } => count,
        };
    }

    /// if the counter is done
    pub fn is_finished(&self) -> bool {
        return match self {
            Self::FINISHED { count: _ } => true,
            _ => false,
        };
    }
    /// takes a reference and decides whether to keep the branch behind this counter
    pub fn keep(&self, count_reference: u16) -> bool {
        return match *self {
            Self::PARTIALLY_FINISHED { count } => count <= count_reference,
            Self::FINISHED { count } => count <= count_reference,
            _ => true,
        };
    }
}
