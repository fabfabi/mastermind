use std::fmt;
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
        return match n_candidates {
            1 => Self::FINISHED { count: 1 },
            // 2 | 3 => Self::PARTIALLY_FINISHED {
            //     count: 2 * n_candidates as u16 - 1,
            // },
            // otherwise return the best estimate how that could be finished
            _ => Self::PENDING {
                count: 2 * n_candidates as u16 - 1,
            },
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
            Self::PENDING { count } => count,
            Self::PARTIALLY_FINISHED { count } => count,
            Self::FINISHED { count } => count,
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
            Self::PENDING { count } => count <= count_reference,
            Self::PARTIALLY_FINISHED { count } => count <= count_reference,
            Self::FINISHED { count } => count <= count_reference,
        };
    }
}
impl fmt::Display for StrategyCounter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Self::PENDING { count } => format!("PENDING({})", count),
            Self::PARTIALLY_FINISHED { count } => format!("PARTIAL({})", count),
            Self::FINISHED { count } => format!("FINISHED({})", count),
        };
        write!(f, "{}", s)
    }
}

#[test]
fn test_str() {
    fn make_str() -> String {
        let a = 2;
        format!("a={}", a)
    }

    println!("{}", make_str())
}
