///trait designed to support the counting function.
/// If all are done -> done
/// if at least one is done or partially finished -> partially finished
/// if all are obsolete -> obsolete
/// otherwise return Unfinished
///
/// Important:
///  * count needs to return: if count < max => Obsolete
pub trait StrategyCounterTrait {
    fn count(self: &Self, max: Option<usize>) -> StrategyCounter;
}

/// function to do the counting and cleaning up of all obsolete paths
pub fn count_and_clean<T>(
    mut candidate_list: Vec<T>,
    max: Option<usize>,
) -> (Vec<T>, StrategyCounter)
where
    T: StrategyCounterTrait,
{
    let mut best_count: u16 = match max {
        Some(number) => number as u16,
        _ => 65535, //biggest u16
    };

    // the loop has to be run twice if there is one that is done
    // run for the first time to find the best value
    for candidate in candidate_list.iter() {
        let new_val = match candidate.count(Some(best_count as usize)) {
            StrategyCounter::Done { count: number } => number,
            StrategyCounter::PartiallyFinished { count: number } => number,
            StrategyCounter::Unfinished { count: _ } => continue, //unfinished ones do not count
            StrategyCounter::Obsolete => continue,
            _ => {
                continue;
            }
        };
        if new_val < best_count {
            best_count = new_val
        }
    }
    // create some test variables that determine the return value
    let mut one_done = false; // if one is done
    let mut all_done = true; // if all are done
    let mut one_partially_finished = false; // if one is partially finished

    // now remove all obsolete ones -> and update the booleans according to the best_count
    candidate_list.retain_mut(|x| match x.count(Some(best_count as usize)) {
        StrategyCounter::Obsolete => false,
        // all others will be kept
        StrategyCounter::Done { count: _ } => {
            one_done = true;
            true
        }
        StrategyCounter::PartiallyFinished { count: _ } => {
            one_partially_finished = true;
            all_done = false;

            true
        }

        _ => {
            all_done = false;
            true
        }
    });
    if candidate_list.len() == 0 {
        return (candidate_list, StrategyCounter::Obsolete);
    } else if all_done {
        return (candidate_list, StrategyCounter::Done { count: best_count });
    } else if one_partially_finished | one_done {
        return (
            candidate_list,
            StrategyCounter::PartiallyFinished { count: best_count },
        );
    }
    let length = candidate_list.len() as u16;
    return (
        candidate_list,
        StrategyCounter::Unfinished {
            count: 2 * length - 1,
        },
    );
}

#[test]
fn test_count_and_clean() {
    // use std::cmp;
    use StrategyCounter;
    /// just a dummy struct to easily fake up some data
    #[derive(Debug, PartialEq)]
    struct S {
        v: u16,
    }
    impl StrategyCounterTrait for S {
        fn count(&self, max: Option<usize>) -> StrategyCounter {
            let matcher = |x| match x {
                0 => StrategyCounter::Obsolete,
                1 => StrategyCounter::Unfinished { count: 1 },
                2..=5 => StrategyCounter::Done { count: x },
                13 => StrategyCounter::Unfinished { count: 13 }, // to check if this will be removed as obsolete
                _ => StrategyCounter::PartiallyFinished { count: x },
            };
            if let Some(number) = max {
                if number < self.v as usize {
                    return matcher(0);
                }
            }
            return matcher(self.v);
        }
    }

    // check if the matcher works correctly
    let s = S { v: 0 };
    assert_eq!(s.count(None), StrategyCounter::Obsolete);
    let s = S { v: 1 };
    assert_eq!(s.count(None), StrategyCounter::Unfinished { count: 1 });
    let s = S { v: 2 };
    assert_eq!(s.count(None), StrategyCounter::Done { count: 2 });
    let s = S { v: 5 };
    assert_eq!(s.count(Some(2)), StrategyCounter::Obsolete);

    // now the magic
    let mut v: Vec<S>;
    let mut c: StrategyCounter;

    // first test
    v = vec![S { v: 0 }, S { v: 1 }, S { v: 2 }];
    (v, c) = count_and_clean(v, None);
    // should remove 0 as obsolete
    assert_eq!(v, vec![S { v: 1 }, S { v: 2 }]);
    assert_eq!(c, StrategyCounter::PartiallyFinished { count: 2 });

    // Second test
    v = vec![S { v: 1 }, S { v: 3 }, S { v: 10 }];
    (v, c) = count_and_clean(v, None);
    // should remove 10 as obsolete and keep the unfinished one
    assert_eq!(v, vec![S { v: 1 }, S { v: 3 }]);
    assert_eq!(c, StrategyCounter::PartiallyFinished { count: 3 });

    // third test
    v = vec![S { v: 0 }, S { v: 3 }, S { v: 4 }];
    (v, c) = count_and_clean(v, None);
    // should keep just the best done one
    assert_eq!(v, vec![S { v: 3 }]);
    assert_eq!(c, StrategyCounter::Done { count: 3 });

    // fourth test -> same as third with a maximum
    v = vec![S { v: 8 }, S { v: 3 }, S { v: 4 }];
    (v, c) = count_and_clean(v, Some(2));
    // should keep just the best done one
    assert_eq!(v, vec![]);
    assert_eq!(c, StrategyCounter::Obsolete);

    // fifth test -> check if the count for "unfinished" works
    v = vec![S { v: 8 }, S { v: 13 }, S { v: 3 }];
    (v, c) = count_and_clean(v, None);
    // should keep just the best done one
    assert_eq!(v, vec![S { v: 3 }]);
    assert_eq!(c, StrategyCounter::Done { count: 3 });
}

/// enum to calculate the number of entries for a strategy
#[derive(Clone, PartialEq, Debug, Copy)]
pub enum StrategyCounter {
    Unfinished { count: u16 },        // counting not yet done
    PartiallyFinished { count: u16 }, // first count available
    Done { count: u16 },              // this strategy path finished already
    Obsolete,                         // This path has more moves than a known path
                                      //End,                              // this is the last node of the strategy
}
impl StrategyCounter {
    pub fn new(n_candidates: usize) -> Self {
        return Self::Unfinished {
            count: 2 * n_candidates as u16,
        };
    }
    pub fn get_count(&self) -> Option<u16> {
        return match self {
            Self::PartiallyFinished { count } => Some(*count),
            Self::Done { count } => Some(*count),
            _ => None,
        };
    }
}

#[test]
fn test_vec_option() {
    let v: Vec<i32> = vec![1, 2, 3, 4];
    let v2: Vec<i32> = v
        .iter()
        .filter_map(|x| if *x > 2 { Some(*x) } else { None })
        .collect();

    println!("done")
}
