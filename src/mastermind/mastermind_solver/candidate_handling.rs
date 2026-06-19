/// old code to use as a reference.
/// The data structure is bad (global hashmap) some aspects might be re-used
/// Going for a directed graph (one node contains all its children)
use crate::mastermind::mastermind_mechanics::get_all_codes;
use crate::mastermind::mastermind_mechanics::grade;
use crate::mastermind::mastermind_mechanics::CodeType;
use crate::mastermind::mastermind_mechanics::ConfigType;
use crate::mastermind::mastermind_mechanics::ResultType;
use crate::mastermind::mastermind_solver::strategy_counter::StrategyCounter;
use std::collections::HashMap;
// use std::hash::Hash;

/// enum to handle the connection to the next level.
///
/// In the first step this is just the hashmap with ResultType -> Vec<CodeType>
/// In the second step this is the hashmap with ResultType -> Vec<CandidateResultType>
///
/// Note: this seems to be a good example for a TypeState pattern.
/// Unfortunately a TypeState Pattern will require a Statemachine to be type-safe
/// which is introducing more boilerplate than it resolves.
/// see: https://users.rust-lang.org/t/how-to-implement-typestate-instead-of-enums-for/140769
enum CandidateResultHashmap {
    NEW(HashMap<ResultType, Vec<CodeType>>),
    PROPAGATED(HashMap<ResultType, CandidateHandlerType>),
}
impl CandidateResultHashmap {
    /// Create the next level of candidates.
    ///
    /// The propagation stops when a CandidateResultHandler is in status done.
    ///
    /// IMPORTANT: This function contains the entire logic and also needs to cover when to stop propagating.
    pub fn create_next_candidates<'a>(&mut self, configuration: &'a ConfigType) {
        match self {
            Self::NEW(hashmap) => {
                let mut hashmap_new: HashMap<ResultType, CandidateHandlerType> = HashMap::new();
                // this is triggering the heavy lifting
                // this could move to a par_iter...
                hashmap.drain().for_each(|(k, v)| {
                    hashmap_new.insert(
                        k,
                        CandidateHandlerType::create_next_candidates(v, &configuration),
                    );
                });

                *self = Self::PROPAGATED(hashmap_new)
            }
            Self::PROPAGATED(hashmap) => {
                hashmap
                    .values_mut()
                    //not entirely sure if that filter is needed
                    .filter(|x| !x.is_done())
                    .for_each(|x| x.propagate_next_level(configuration));
            }
        }
    }
    /// verify if the hashmap contains the other_result already
    pub fn contains(&self, other_result: &ResultType) -> bool {
        return match self {
            Self::NEW(hm) => hm.contains_key(other_result),
            Self::PROPAGATED(hm) => hm.contains_key(other_result),
        };
    }
    /// return the number of results
    pub fn number_of_next_candidates(&self) -> usize {
        return match self {
            Self::NEW(hm) => hm.keys().len(),
            Self::PROPAGATED(hm) => hm.keys().len(),
        };
    }

    /// number of the candidates for one specific result
    pub fn number_of_candidates_per_result(&self, result: &ResultType) -> usize {
        if let Self::NEW(hm) = self {
            if let Some(entries) = hm.get(result) {
                return entries.len();
            }
        }
        return 0;
    }

    /// compare two CandidateResultHashmaps
    /// These are defined same if there are same numbers of candidates behind the same results
    pub fn eq(&self, other: &Self) -> bool {
        if let Self::NEW(hm) = self {
            if !other.number_of_next_candidates() == self.number_of_next_candidates() {
                return false;
            }
            for (result, entries) in hm {
                if entries.len() != other.number_of_candidates_per_result(&result) {
                    return false;
                }
            }
        }
        return true;
    }
    // Might be a bit of an overkill to define a generic function to execute closures on a Hashmap
    // fn exec_closure<T>(self, closure: Box<dyn Fn(HashMap<ResultType, T>) >){
    //     match self {
    //         Self::NEW(hm) => closure(hm),
    //         Self::PROPAGATED(hm) => closure(hm),
    //     }
    // }

    /// show the output
    pub fn show_details(&self, indentation: usize) {
        match self {
            Self::NEW(hm) => {
                println!(
                    "{}{} CandidateResultHashmap::NEW",
                    " ".repeat(indentation),
                    indentation
                );
                for (k, v) in hm.iter() {
                    println!(
                        "{}{} {}{}",
                        " ".repeat(indentation),
                        indentation,
                        k,
                        v.len()
                    );
                }
            }
            Self::PROPAGATED(hm) => {
                println!(
                    "{}{} CandidateResultHashmap::PROPAGATED",
                    " ".repeat(indentation),
                    indentation
                );
                for (k, v) in hm.iter() {
                    println!("{}{} {}", " ".repeat(indentation), indentation, k,);
                    v.show_details(indentation + 1);
                }
            }
        }
    }

    // pub fn show_details(&self, indentation: usize) {}

    /// function to count the number of tries of one candidates.
    ///
    /// The difference to the count function of the CandidateHandlerType:
    /// * Adds `number_of_candidates` to the total count. (one try for each candidate)
    /// * PARTIALLY_FINISHED if all children can be FINISHED in the next loop.
    /// * FINISHED if all children are FINISHED.
    pub fn count(&mut self, number_of_candidates: usize) -> StrategyCounter {
        // closure to check the finished path
        let check_finished = |hm_keys_length: usize, other_option: StrategyCounter| {
            if hm_keys_length == number_of_candidates {
                // if there is just one candidate left in this group
                // note: the ResultHandlerType ensures, that the last candidate is also taken
                if number_of_candidates == 1 {
                    return StrategyCounter::FINISHED { count: 1 };
                }

                return StrategyCounter::PARTIALLY_FINISHED {
                    count: 2 * number_of_candidates as u16 - 1, // exact number is clear
                };

                // the configuration is not available here and the difference is only one...
                // let value: u16 = number_of_candidates as u16;
                // // check if the solution was found and return the number of steps
                // return match self.contains(&ResultType::is_done(&self, configuration)) {
                //     true => StrategyCounter::PartiallyFinished {
                //         count: 2 * value - 1, // exact number is clear
                //     },
                //     _ => StrategyCounter::PartiallyFinished {
                //         count: 2 * value, // worst case scenario, since the solution is not part of this step
                //     },
                // };
            }
            return other_option;
        };

        return match self {
            // new means the children are not yet counted
            Self::NEW(hm) => {
                check_finished(hm.keys().len(), StrategyCounter::new(number_of_candidates))
            }
            Self::PROPAGATED(hm) => {
                // this is the actual counting logic
                // -> sum the count of all children
                let count_children = hm
                    .values_mut()
                    // this triggers the update of all children
                    .map(|x| x.count().get_count_estimate())
                    .sum::<u16>();

                // return DONE if all are DONE otherwise unfinished
                // this is detected by the closure for the return type
                let get_return_type = |count_total: u16| {
                    if hm.values().all(|x| x.is_done()) {
                        return StrategyCounter::FINISHED { count: count_total };
                    }
                    StrategyCounter::PENDING { count: count_total }
                };

                return check_finished(
                    hm.keys().len(),
                    get_return_type(
                        // the total count is the sum of all other counts
                        count_children
                            // sum all future counts + the current count (i.e. one input per candidate)
                            + number_of_candidates as u16,
                    ),
                );
            }
        };
    }
}

/// class to handle the results of one candidate
///
/// the main parts of the logic is contained within CandidateResultHashmap
/// This struct bridges to CandidateResultHashmap which acts as a state machine
// #[derive(Clone)]
struct CandidateResultType {
    result_hashmap: CandidateResultHashmap,
    candidate: CodeType,
    counter_stored: StrategyCounter,
    number_of_candidates: usize,
}
impl CandidateResultType {
    /// create a new result.
    /// Note, this is done plenty of times, and non-valueadding ones are scrapped
    fn new(guesses: &Vec<CodeType>, solution: &CodeType) -> Self {
        let mut map: HashMap<ResultType, Vec<CodeType>> = HashMap::new();

        for guess in guesses.iter() {
            let result = grade(&guess, &solution);
            map.entry(result)
                .or_insert_with(|| Vec::<CodeType>::new())
                .push(guess.clone()); // would this work without the "clone" --> NO?
        }

        let n_candidates = guesses.len();
        return CandidateResultType {
            result_hashmap: CandidateResultHashmap::NEW(map),
            candidate: solution.clone(),
            counter_stored: StrategyCounter::new(n_candidates),
            number_of_candidates: n_candidates,
        };
    }

    /// propagate to the next level
    ///
    /// On purpose this does not contain any logic in order to concentrate
    /// the logic within CandidateResultHashmap::create_next_candidates
    fn propagate_next_level(&mut self, configuration: &ConfigType) {
        self.result_hashmap.create_next_candidates(&configuration);
    }

    ///returns if a given result is present in that hashmap
    fn contains(&self, other_result: &ResultType) -> bool {
        self.result_hashmap.contains(other_result)
    }

    ///returns the overall number of different results
    fn num_results(&self) -> usize {
        return self.result_hashmap.number_of_next_candidates();
    }

    ///returns the number of entries for a given result
    fn num_entries(&self, result: &ResultType) -> usize {
        self.result_hashmap.number_of_candidates_per_result(result)
    }

    /// executing the count and update the counter
    pub fn count(&mut self) -> &StrategyCounter {
        self.counter_stored = self.result_hashmap.count(self.number_of_candidates);

        return &self.counter_stored;
    }

    /// show a high-level summary
    fn show(&self, indentation: usize) {
        println!(
            "{}{} Candidate: '{}' with {} groups",
            " ".repeat(indentation),
            indentation,
            self.candidate,
            self.result_hashmap.number_of_next_candidates()
        );
    }

    fn show_details(&self, indentation: usize) {
        self.show(indentation);
        self.result_hashmap.show_details(indentation);
    }

    ///checks if two result_handler are equal (i.e. same results and same number of entries per resulg)
    fn eq(&self, other: &Self) -> bool {
        return self.result_hashmap.eq(&other.result_hashmap);
    }
}

#[test]
fn test_result_handler_basics() {
    let guesses = vec![
        CodeType::new(vec![1, 2, 3, 4]),
        CodeType::new(vec![1, 1, 1, 1]),
        CodeType::new(vec![2, 2, 2, 2]),
        CodeType::new(vec![3, 3, 3, 3]),
        CodeType::new(vec![4, 4, 4, 4]),
    ];

    let solution = CodeType::new(vec![1, 2, 3, 4]);
    let result_handler = CandidateResultType::new(&guesses, &solution);

    assert!(result_handler.contains(&ResultType::new(4, 0)));
    assert!(result_handler.contains(&ResultType::new(1, 0)));
    assert!(!result_handler.contains(&ResultType::new(1, 1)));
    assert_eq!(result_handler.num_results(), 2);

    assert_eq!(result_handler.num_entries(&ResultType::new(4, 0)), 1);
    assert_eq!(result_handler.num_entries(&ResultType::new(1, 0)), 4);

    // assert!(matches!(
    //     result_handler.count(None),
    //     StrategyCounter::Unfinished { count: 9 }
    // ));

    // example that should not be equal to above
    let guesses_neq = vec![
        CodeType::new(vec![1, 2, 3, 4]),
        CodeType::new(vec![2, 2, 2, 2]),
        CodeType::new(vec![3, 3, 3, 3]),
        CodeType::new(vec![4, 4, 4, 4]),
    ];
    let result_handler_neq = CandidateResultType::new(&guesses_neq, &solution);
    assert!(!result_handler.eq(&result_handler_neq));

    //another example that should be equal to the result handler
    let guesses_eq = vec![
        CodeType::new(vec![1, 2, 3, 4]),
        CodeType::new(vec![2, 2, 2, 2]),
        CodeType::new(vec![3, 3, 3, 3]),
        CodeType::new(vec![4, 4, 4, 4]),
        CodeType::new(vec![0, 2, 0, 0]),
    ];
    let result_handler_eq = CandidateResultType::new(&guesses_eq, &solution);
    assert!(result_handler.eq(&result_handler_eq));
    assert!(result_handler.candidate.eq(&solution));

    //now testing also the count -> for only one result
    let guesses_cnt1 = vec![CodeType::new(vec![1, 2, 3, 4])];
    let mut result_handler_cnt1 = CandidateResultType::new(&guesses_cnt1, &solution);

    assert_eq!(
        *result_handler_cnt1.count(),
        StrategyCounter::FINISHED { count: 1 }
    );

    //now testing also the count -> for two results
    let guesses_cnt2 = vec![
        CodeType::new(vec![1, 2, 3, 4]),
        CodeType::new(vec![1, 2, 3, 5]),
    ];
    let mut result_handler_cnt2 = CandidateResultType::new(&guesses_cnt2, &solution);

    // note: this should become 3 but I did not want to implement into CandidateResultHashmap.count to check
    // whether the finished solution is contained. If this changes -> 3
    assert_eq!(
        *result_handler_cnt2.count(),
        StrategyCounter::PARTIALLY_FINISHED { count: 4 }
    );

    // // check if the Obsolete path works --> Obsolete is removed
    // assert_eq!(
    //     result_handler_cnt2.count(),
    //     StrategyCounter::Obsolete // not better than the other one -> obsolete
    // );
}

#[test]
fn test_candidate_handler_propagation2() {
    let configuration = &ConfigType {
        columns: 3,
        colors: 4,
    };
    // trivial case.
    // if you enter all codes after another, this ends up in 3 rounds and 6 total tries
    let guesses = vec![
        CodeType::new(vec![0, 0, 1]),
        CodeType::new(vec![0, 1, 2]),
        CodeType::new(vec![3, 0, 0]),
    ];
    let mut result_handler = CandidateHandlerType::create_next_candidates(guesses, &configuration);
    result_handler.count();
    result_handler.show_details(0);

    result_handler.propagate_next_level(configuration);
    result_handler.count();
    result_handler.show_details(0);

    assert_eq!(
        result_handler.counter_stored,
        StrategyCounter::PARTIALLY_FINISHED { count: 5 }
    );

    result_handler.propagate_next_level(configuration);
    result_handler.count();
    result_handler.show_details(0);
    assert_eq!(
        result_handler.counter_stored,
        StrategyCounter::FINISHED { count: 5 }
    );
}
#[test]
fn test_candidate_handler_propagation() {
    let configuration = &ConfigType {
        columns: 3,
        colors: 5,
    };
    let guesses = vec![
        CodeType::new(vec![1, 1, 1]),
        CodeType::new(vec![2, 2, 2]),
        CodeType::new(vec![3, 3, 3]),
        CodeType::new(vec![4, 4, 4]),
    ];
    let mut result_handler = CandidateHandlerType::create_next_candidates(guesses, &configuration);
    assert_eq!(
        result_handler.counter_stored,
        StrategyCounter::PENDING { count: 7 }
    );
    // update the counter
    result_handler.count();
    // result_handler.show_details();

    assert_eq!(
        result_handler.counter_stored,
        StrategyCounter::PARTIALLY_FINISHED { count: 7 }
    );
    result_handler.propagate_next_level(configuration);
    // result_handler.show_details();
    let a = 2;
    assert_eq!(
        *result_handler.count(),
        StrategyCounter::FINISHED { count: 6 }
    );
    result_handler.propagate_next_level(configuration);
    // result_handler.show_details();
    let a = 2;
    assert_eq!(
        *result_handler.count(),
        StrategyCounter::FINISHED { count: 6 }
    );
}

/// class to identify the next inputs to test
/// After entering a list of candidates (that are open), this class finds all possible next inputs
struct CandidateHandlerType {
    candidate_list: Vec<CandidateResultType>,
    counter_stored: StrategyCounter,
    number_of_candidates: u16, // TBD: needed for the counting logic?
}
impl CandidateHandlerType {
    /// create a candidate handler for a completely new game
    fn new(configuration: &ConfigType) -> Self {
        let candidates = get_all_codes(configuration);
        return CandidateHandlerType::create_next_candidates(candidates, configuration);
    }
    ///creates the next level of candidates. This function triggers the heavy lifting
    fn create_next_candidates(candidates: Vec<CodeType>, configuration: &ConfigType) -> Self {
        let mut result = CandidateHandlerType {
            candidate_list: Vec::new(),
            counter_stored: StrategyCounter::new(candidates.len()),
            number_of_candidates: candidates.len() as u16,
        };
        // if there are only a few candidates left, no more grading needed
        // definitely works for 1 and 2, should also work for other small numbers TO BE CHECKED!!!
        if candidates.len() <= 2 {
            let candidate = candidates[0].clone();
            let new_candidate_result =
                CandidateResultType::new(&vec![candidate.clone()], &candidate);
            result.add(new_candidate_result);
            return result;
        }
        // this one should move to become either a static variable
        // or some static method from configuration so this is loaded once and then just returned
        let ALL_CANDIDATES = get_all_codes(configuration);

        for code in ALL_CANDIDATES.iter() {
            let new_candidate_result = CandidateResultType::new(&candidates, code);
            // no benefit in checking candidate that does not increase information
            if new_candidate_result.num_results() == 1 {
                continue;
            }
            if result.contains_similar(&new_candidate_result) {
                continue;
            }

            result.add(new_candidate_result)
        }

        return result;
    }

    /// return the number of candidates identified for the next input
    fn number_of_candidates_next_input(&self) -> usize {
        self.candidate_list.len()
    }

    fn propagate_next_level(&mut self, configuration: &ConfigType) {
        self.candidate_list
            .iter_mut()
            .for_each(|x| x.propagate_next_level(&configuration))
    }
    /// check if a similar CandidateResult is already found
    fn contains_similar(&self, other_candidate_result: &CandidateResultType) -> bool {
        for code in self.candidate_list.iter() {
            if code.eq(&other_candidate_result) {
                return true;
            }
        }

        return false;
    }

    /// initiate the counting logic that also updates the status
    fn count_initiate(&mut self) {
        self.count();
    }

    /// global counting Logic:
    /// 1) count the number of tries and update the Status (calling top-down and finishing bottom up)
    /// 2) cut away the unneccesary branches (top-down)
    ///
    /// For the CandidateHandlerType: cut away the branches that are worse than already finished branches
    ///
    /// The difference to the count function of the CandidateResultHashmap:
    /// * does not add to the total count
    /// * PARTIALLY_FINISHED if one child is FINISHED
    /// * FINISHED if all children are FINISHED (or removed)
    fn count(&mut self) -> &StrategyCounter {
        // update the counter of all children and retrieve the best count
        // first check: find finished paths
        if let Some(count_best_finished) = self
            .candidate_list
            .iter_mut()
            // -> this triggers the update of the children
            .filter_map(|x| x.count().get_count_finished())
            .min()
        {
            // chop-off all candidates that exceed the best count
            self.candidate_list
                .retain(|x| x.counter_stored.keep(count_best_finished));

            // and set the count storer
            if self
                .candidate_list
                .iter()
                .all(|x| x.counter_stored.is_finished())
            {
                self.counter_stored = StrategyCounter::FINISHED {
                    count: count_best_finished,
                };
            } else {
                self.counter_stored = StrategyCounter::PARTIALLY_FINISHED {
                    count: count_best_finished,
                };
            }
        // no finished paths -> find pending ones
        } else if let Some(count_best) = self
            .candidate_list
            .iter()
            .map(|x| x.counter_stored.get_count_estimate())
            .min()
        {
            self.counter_stored = StrategyCounter::PARTIALLY_FINISHED { count: count_best };
        }
        return &self.counter_stored;
    }

    /// return the number of candidates found
    fn len(&self) -> usize {
        return self.candidate_list.len();
    }

    /// adds a candidate to the list
    fn add(&mut self, candidate: CandidateResultType) {
        self.candidate_list.push(candidate)
    }

    /// if this is the last entry, i.e. only one candidate left
    fn is_done(&self) -> bool {
        return self.candidate_list.len() == 1;
    }

    fn show(&self, indentation: usize) {
        println!(
            "{}{} CandidateHandler with {} candidates",
            " ".repeat(indentation),
            indentation,
            self.candidate_list.len()
        );
    }
    fn show_details(&self, indentation: usize) {
        self.show(indentation);
        for candidate in &self.candidate_list {
            candidate.show_details(indentation + 1);
        }
    }
}

#[test]
fn test_candidatehandlertype_basic() {
    // this is just the basic testing without the propagation
    let config = ConfigType {
        colors: 2,
        columns: 2,
    };
    let cht_one =
        CandidateHandlerType::create_next_candidates(vec![CodeType::new(vec![0, 0])], &config);

    assert_eq!(cht_one.len(), 1);
    assert!(cht_one.is_done());

    let mut cht = CandidateHandlerType::create_next_candidates(
        vec![
            CodeType::new(vec![0, 0]),
            CodeType::new(vec![1, 0]),
            CodeType::new(vec![0, 1]),
            CodeType::new(vec![1, 1]),
        ],
        &config,
    );

    // there should be only two candidates.
    // Same color (20, 10x2, 00 as results where 10x2 means 1 correct ones and 0 correct positions with 2 codes)
    // or different color (20 10x2 02)
    assert_eq!(cht.len(), 2);
    cht.count();
    cht.show_details(0);
    assert_eq!(
        cht.counter_stored,
        StrategyCounter::PARTIALLY_FINISHED { count: 7 }
    );

    //////////////////////////////////////////
    // now with only three as input
    let mut cht = CandidateHandlerType::create_next_candidates(
        vec![
            CodeType::new(vec![0, 0]),
            CodeType::new(vec![1, 0]),
            CodeType::new(vec![0, 1]),
        ],
        &config,
    );
    // now there will be three candidates:
    // 00 will get 20 and 10x2
    // 10 or 01 will get 20, 10, 02 -> partially finished!
    // 11 will get 00 and 10 x 2
    assert_eq!(cht.len(), 3);

    cht.count();
    assert_eq!(
        *cht.count(),
        StrategyCounter::PARTIALLY_FINISHED { count: 6 }
    );
    // Note: this could also be considered as PartiallyFinished since one of the paths is finished
    // 10 will calculate a partial finish at 5

    //////////////////////////////////////////
    //  just a check on the high-level
    let config64 = ConfigType {
        colors: 6,
        columns: 4,
    };
    // still needed?
    let cht = StrategyHandlerType::new(&config64);
    assert_eq!(cht.candidate_handling.number_of_candidates_next_input(), 5);
}

/// High level type to handle the solving process
pub struct StrategyHandlerType<'a> {
    configuration: &'a ConfigType,
    candidate_handling: CandidateHandlerType,
}
impl StrategyHandlerType<'_> {
    pub fn new<'a>(config: &'a ConfigType) -> StrategyHandlerType<'a> {
        StrategyHandlerType {
            configuration: config,
            candidate_handling: CandidateHandlerType::new(&config),
        }
    }

    pub fn solve(&mut self) {}
}
