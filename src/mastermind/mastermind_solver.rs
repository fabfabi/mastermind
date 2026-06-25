/// old code to use as a reference.
/// The data structure is bad (global hashmap) some aspects might be re-used
/// Going for a directed graph (one node contains all its children)
use crate::mastermind::mastermind_mechanics::grade;
use crate::mastermind::mastermind_mechanics::CodeType;
use crate::mastermind::mastermind_mechanics::ConfigType;
use crate::mastermind::mastermind_mechanics::ResultType;

// use crate::mastermind::mastermind_solver::candidate_handling::StrategyExecutionType::GUESS;

use rayon::prelude::*;
use simplelog::*;
use std::collections::HashMap; // needed for logging

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
    /// and another shot for clearing all the unfinished ones.
    ///
    /// Note: The count should only increase since this is the BoB Case.
    pub fn new(n_candidates: usize) -> Self {
        return match n_candidates {
            1 => Self::FINISHED {
                count: n_candidates as u16,
            }, // still needs the conversion to "propagated"
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

/// enum to handle the connection to the next level.
///
/// In the first step this is just the hashmap with ResultType -> Vec<CodeType>
/// In the second step this is the hashmap with ResultType -> Vec<CandidateResultType>
///
/// Note: this seems to be a good example for a TypeState pattern.
/// Unfortunately a TypeState Pattern will require a Statemachine to be type-safe
/// which is introducing more boilerplate than it resolves.
/// see: https://users.rust-lang.org/t/how-to-implement-typestate-instead-of-enums-for/140769
enum CandidateResultHashMap {
    NEW(HashMap<ResultType, Vec<CodeType>>),
    PROPAGATED(HashMap<ResultType, CandidateHandlerType>),
}
impl CandidateResultHashMap {
    /// Create the next level of candidates.
    ///
    /// The propagation stops when a CandidateResultHandler is in status done.
    ///
    /// IMPORTANT: This function contains the entire logic and also needs to cover when to stop propagating.
    pub fn create_next_candidates<'a>(&mut self, configuration: &'a ConfigType) {
        match self {
            Self::NEW(hashmap) => {
                // only update if there are 2 or more candidates. For one candidate, no further input is needed
                if hashmap.len() > 1 {
                    let mut hashmap_new: HashMap<ResultType, CandidateHandlerType> = HashMap::new();
                    // this is triggering the heavy lifting
                    // this could move to a par_iter...
                    hashmap
                        .drain()
                        //exclude paths that are detected as finished. No more input needed.
                        .filter(|(k, _)| *k != configuration.finished_result())
                        .for_each(|(k, v)| {
                            hashmap_new.insert(
                                k,
                                CandidateHandlerType::create_next_candidates(v, &configuration),
                            );
                        });

                    *self = Self::PROPAGATED(hashmap_new)
                }
            }
            Self::PROPAGATED(hashmap) => {
                hashmap
                    .values_mut()
                    //not entirely sure if that filter is needed
                    .filter(|x| !x.is_finished())
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
    /// propagate to cut away unneccessary branches
    pub fn prune(&mut self) {
        if let Self::PROPAGATED(hm) = self {
            hm.values_mut().for_each(|x| x.prune());
        }
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
            return 0;
        } else {
            panic!("This logic should only be checked for CandidateResultHashmap::NEW when creating new Hashmaps");
        }
    }

    /// return the handler of a certain result
    pub fn get_handler(&self, result: &ResultType) -> Option<&CandidateHandlerType> {
        return match self {
            Self::NEW(hm) => {
                for (k, v) in hm {
                    println!("{}", k);
                    for c in v {
                        println!("    {}", c)
                    }
                }
                panic!("Solution is not yet found, there are more levels")
            }
            Self::PROPAGATED(hm) => hm.get(result),
        };
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

    /// show the output
    pub fn show_details(&self, indentation: usize, head: Option<String>) {
        let head_str = format!(
            "{}{}: {}",
            " ".repeat(indentation * 3),
            indentation,
            head.unwrap_or("".into())
        );
        match self {
            Self::NEW(hm) => {
                for (k, v) in hm.iter() {
                    if v.len() == 1 {
                        info!("{} => {} -> {} DONE", head_str, k, v.first().unwrap());
                    } else {
                        info!("{} => {} -> {} codes", head_str, k, v.len());
                    }
                }
            }
            Self::PROPAGATED(hm) => {
                for (k, v) in hm.iter() {
                    info!("{} => {} @ {}", head_str, k, v.counter_stored);
                    v.show_details(indentation + 1);
                }
            }
        }
    }

    /// function to count the number of tries of one candidates.
    ///
    /// The difference to the count function of the CandidateHandlerType:
    /// * Adds `number_of_candidates` to the total count. (one try for each candidate)
    /// * PARTIALLY_FINISHED if all children can be FINISHED in the next loop.
    /// * FINISHED if all children are FINISHED.
    pub fn count(&mut self, number_of_candidates: usize) -> StrategyCounter {
        // closure to check the finished path
        let check_finished = |hm_keys_length: usize, other_option: StrategyCounter| {
            // if a finish was detected already
            if other_option.is_finished() {
                return other_option;
            }
            if hm_keys_length == number_of_candidates {
                // if there is just one candidate left in this group
                // note: the ResultHandlerType ensures, that the last candidate is also taken
                if hm_keys_length == 1 {
                    // note that 0 means that the solution was already found the turn before and
                    // and there is no more input needed.
                    return StrategyCounter::FINISHED {
                        count: hm_keys_length as u16,
                    };
                }
                return StrategyCounter::PARTIALLY_FINISHED {
                    count: 2 * number_of_candidates as u16 - 1, // exact number is clear
                };
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

                // only for debugging purposes to see all the details
                // hm.values().for_each(|x| x.show_details(0));

                // return DONE if all are DONE otherwise unfinished
                // this is detected by the closure for the return type
                let get_return_type = |count_total: u16| {
                    // debug!(
                    //     "next_finished: {}, # keys {}",
                    //     hm.values().all(|x| x.is_finished()),
                    //     hm.keys().len()
                    // );
                    if hm.values().all(|x| x.is_finished()) {
                        debug!("returning 'Finished'");
                        return StrategyCounter::FINISHED { count: count_total };
                    } else {
                        return StrategyCounter::PENDING { count: count_total };
                    }
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
    result_hashmap: CandidateResultHashMap,
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
            result_hashmap: CandidateResultHashMap::NEW(map),
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
        if let StrategyCounter::FINISHED { .. } = self.counter_stored {
            // do nothing
        } else {
            self.result_hashmap.create_next_candidates(&configuration)
        }
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

    fn prune(&mut self) {
        self.result_hashmap.prune()
    }

    /// executing the count and update the counter
    pub fn count(&mut self) -> &StrategyCounter {
        debug!("counting {} --> start", self.candidate);
        self.counter_stored = self.result_hashmap.count(self.number_of_candidates);
        debug!(
            "counting {}, --> finished: {}",
            self.candidate, self.counter_stored
        );
        return &self.counter_stored;
    }

    /// return the candidate that is entered at this step
    fn get_guess(&self) -> CodeType {
        self.candidate.clone()
    }

    /// show a high-level summary
    fn show(&self, indentation: usize) {
        info!(
            "{}{}: Candidate: '{}' with {} groups",
            " ".repeat(indentation),
            indentation,
            self.candidate,
            self.result_hashmap.number_of_next_candidates()
        );
    }

    fn show_details(&self, indentation: usize) {
        // self.show(indentation);
        self.result_hashmap
            .show_details(indentation, Some(self.candidate.clone().into()));
    }

    ///checks if two result_handler are equal (i.e. same results and same number of entries per resulg)
    fn eq(&self, other: &Self) -> bool {
        return self.result_hashmap.eq(&other.result_hashmap);
    }
}

/// class to identify the next inputs to test
/// After entering a list of candidates (that are open), this class finds all possible next inputs
struct CandidateHandlerType {
    candidate_list: Vec<CandidateResultType>,
    counter_stored: StrategyCounter,
}
impl CandidateHandlerType {
    /// create a candidate handler for a completely new game
    fn new(configuration: &ConfigType) -> Self {
        if let Some(candidates) = configuration.get_all_codes() {
            return CandidateHandlerType::create_next_candidates(candidates.clone(), configuration);
        } else {
            panic!("configuration needs to be initialized")
        }
    }
    ///creates the next level of candidates. This function triggers the heavy lifting
    fn create_next_candidates(candidates: Vec<CodeType>, configuration: &ConfigType) -> Self {
        let mut result = CandidateHandlerType {
            candidate_list: Vec::new(),
            counter_stored: StrategyCounter::new(candidates.len()),
        };
        // if there are only a few candidates left, no more grading needed
        // definitely works for 1 and 2, should also work for other small numbers TO BE CHECKED!!!
        if candidates.len() <= 2 {
            let candidate = candidates[0].clone();
            let new_candidate_result = CandidateResultType::new(&candidates, &candidate);
            result.add(new_candidate_result);
            return result;
        }
        // unwrap should be possible since the new function checks the initialization
        let candidates_all = configuration.get_all_codes().unwrap();

        for code in candidates_all.iter() {
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

    fn propagate_next_level(&mut self, configuration: &ConfigType) {
        self.candidate_list
            .par_iter_mut()
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

    fn prune(&mut self) {
        if let StrategyCounter::FINISHED { .. } = self.counter_stored {
            // take only the first candidate. All others are comparable
            self.candidate_list.truncate(1);
            // and propagate the pruning
            self.candidate_list.iter_mut().for_each(|x| x.prune());
        } else {
            panic!("Too early to prune. Needs to finish first.")
        }
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

            // Note: keeping only the first candidate will not find the optimal strategy. truncation has to be done at the end.

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

    /// get the next guess
    fn get_candidate(&self) -> &CandidateResultType {
        self.candidate_list.first().unwrap()
    }

    /// if this path is finished.
    ///
    /// There are two ways to detect:
    /// * if there is just one candidate left (for later steps)
    /// * via the StrategyCounter which can also propagate the status upwards.
    fn is_finished(&self) -> bool {
        return match self.counter_stored {
            StrategyCounter::FINISHED { count: _ } => true,
            _ => false,
        };
        // return self.candidate_list.len() == 1;
    }
    /// initiate to show all the details
    pub fn show(&self) {
        info!("{}", "#".repeat(30),);
        info!("Candidate Handler count: {}", self.counter_stored);

        for candidate in &self.candidate_list {
            info!(
                "Next candidate: {} @ {}",
                candidate.get_guess(),
                candidate.counter_stored
            );
            candidate.show_details(0);
        }
    }
    /// Function to be called recursively to show the details
    fn show_details(&self, indentation: usize) {
        // self.show(indentation);
        for candidate in &self.candidate_list {
            candidate.show_details(indentation);
        }
    }
}
/// State machine to handle the execution of a strategy
enum StrategyExecutionType<'a> {
    GUESS {
        handler_candidate: &'a CandidateHandlerType,
    },
    RESPOND {
        handler_result: &'a CandidateResultHashMap,
    },
    FINISHED,
}
impl<'a> StrategyExecutionType<'a> {
    pub fn new(candidates: &'a CandidateHandlerType) -> Self {
        if !candidates.counter_stored.is_finished() {
            panic!("Need a CandidateHandlerType that is FINISHED.")
        }
        StrategyExecutionType::GUESS {
            handler_candidate: &candidates,
        }
    }
    /// retrieves the next guess and changes the state to RESPOND
    pub fn get_guess(&mut self) -> CodeType {
        return match self {
            Self::GUESS {
                handler_candidate: handler,
            } => {
                let candidate_result = handler.get_candidate();

                let guess = candidate_result.get_guess();

                *self = StrategyExecutionType::RESPOND {
                    handler_result: &candidate_result.result_hashmap,
                };
                guess
            }
            Self::RESPOND { .. } => panic!("Guess was already returned"),
            Self::FINISHED => panic!("Execution already Finished"),
        };
    }
    /// enter the result of a guess and changes the state to GUESS or FINISHED
    pub fn enter_result(&mut self, result: ResultType) {
        match self {
            Self::GUESS { .. } => panic!("Result of last guess was already returned"),
            Self::RESPOND { handler_result } => {
                if let Some(cht) = handler_result.get_handler(&result) {
                    *self = StrategyExecutionType::GUESS {
                        handler_candidate: cht,
                    }
                } else {
                    *self = StrategyExecutionType::FINISHED
                }
            }
            Self::FINISHED => panic!("Execution already Finished"),
        }
    }
    /// if a StrategyExecutionType has finished
    pub fn is_finished(&self) -> bool {
        return match self {
            Self::FINISHED => true,
            _ => false,
        };
    }
}

/// High level type to handle the solving process
pub struct StrategyHandlerType<'a> {
    configuration: &'a ConfigType,
    handler_candidates: CandidateHandlerType,
}
impl StrategyHandlerType<'_> {
    pub fn new<'a>(config: &'a ConfigType) -> StrategyHandlerType<'a> {
        info!("Identify the strategy for {}", &config);
        StrategyHandlerType {
            configuration: config,
            handler_candidates: CandidateHandlerType::new(&config),
        }
    }

    /// cut away unneecessary branches
    pub fn prune(&mut self) {
        self.handler_candidates.prune()
    }

    /// solve to find the best strategy
    pub fn solve(&mut self) {
        info!("Solving: Brute force to find the best strategy");
        while !self.handler_candidates.counter_stored.is_finished() {
            info!("propagating next level");
            self.handler_candidates
                .propagate_next_level(self.configuration);
            self.handler_candidates.count();
        }
        self.prune();
    }

    /// test function to solve and verify the different expected count estimates
    pub fn test_solve(&mut self, counter_expectation: Vec<StrategyCounter>) {
        let count_last = counter_expectation.last().unwrap().clone();
        for counter in counter_expectation {
            // default setting -> expected to be solved in best case 7
            self.handler_candidates.show();
            assert_eq!(self.handler_candidates.counter_stored, counter);

            self.handler_candidates
                .propagate_next_level(self.configuration);
            self.handler_candidates.count();
        }
        // check the last one - nothing changes once a Candidatehandler is FINISHED.
        assert_eq!(self.handler_candidates.counter_stored, count_last);

        assert!(self.handler_candidates.is_finished())
    }

    /// verify the count of the strategy
    pub fn verify(&self) {
        let candidates_all = self.configuration.get_all_codes().unwrap();

        let mut count = 0;
        for solution in candidates_all {
            debug!("Solving {}", solution);
            // initiate strategy
            let mut strategy = StrategyExecutionType::new(&self.handler_candidates);
            for _ in 0..100 {
                count += 1;
                let guess = strategy.get_guess();
                let result = guess.grade(&solution);
                if result.is_finished(&self.configuration) {
                    debug!("guessing {} -> {} FINISHED", guess, result);
                    break;
                } else {
                    debug!("guessing {} -> {}", guess, result);
                }
                strategy.enter_result(result);
            }
        }
        info!(
            "Strategy checked: Expected {} and returned {} guesses",
            self.handler_candidates.counter_stored.get_count_estimate(),
            count,
        );
        assert_eq!(
            self.handler_candidates.counter_stored.get_count_estimate(),
            count
        );
    }

    /// show the entire strategy
    pub fn show(&self) {
        info!("Exporting the strategy");
        self.handler_candidates.show();
    }
}
/////////////////////////////////////////////////////////////////////////////////////////////////
/// Testing begins
/////////////////////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod test_candidate_handling {
    use super::*;

    /// initiate the logger to have logging for the tests
    fn logger_initiate(level: Option<LevelFilter>) {
        let lvl = level.unwrap_or(LevelFilter::Info);
        if let Err(e) = CombinedLogger::init(vec![TermLogger::new(
            lvl,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )]) {
            // not an issue if this errors. It just means it has already been initiated by another
            // task
            println!("Setting the logger errored{}", e);
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
            StrategyCounter::PARTIALLY_FINISHED { count: 3 }
        );

        // // check if the Obsolete path works --> Obsolete is removed
        // assert_eq!(
        //     result_handler_cnt2.count(),
        //     StrategyCounter::Obsolete // not better than the other one -> obsolete
        // );
    }

    #[test]
    fn test_candidatehandlertype_basic() {
        // this is just the basic testing without the propagation
        let config = ConfigType::new_extended(2, 2);
        let cht_one =
            CandidateHandlerType::create_next_candidates(vec![CodeType::new(vec![0, 0])], &config);

        assert_eq!(cht_one.len(), 1);
        assert!(cht_one.is_finished());

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
        // cht.show_details(0);
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
            StrategyCounter::PARTIALLY_FINISHED { count: 5 }
        );
        // Note: this could also be considered as PartiallyFinished since one of the paths is finished
        // 10 will calculate a partial finish at 5
    }

    #[test]
    fn test_candidate_handler_propagation() {
        let configuration = &ConfigType::new_extended(5, 3);
        let guesses = vec![
            CodeType::new(vec![1, 1, 1]),
            CodeType::new(vec![2, 2, 2]),
            CodeType::new(vec![3, 3, 3]),
            CodeType::new(vec![4, 4, 4]),
        ];
        let mut handler_candidate =
            CandidateHandlerType::create_next_candidates(guesses, &configuration);
        assert_eq!(
            handler_candidate.counter_stored,
            StrategyCounter::PENDING { count: 7 }
        );
        // update the counter
        handler_candidate.count();
        // handler_candidate.show();

        assert_eq!(
            handler_candidate.counter_stored,
            StrategyCounter::PARTIALLY_FINISHED { count: 7 }
        );
        handler_candidate.propagate_next_level(configuration);
        // handler_candidate.show();

        assert_eq!(
            *handler_candidate.count(),
            StrategyCounter::PARTIALLY_FINISHED { count: 9 }
        );
        handler_candidate.propagate_next_level(configuration);
        handler_candidate.show();

        assert_eq!(
            *handler_candidate.count(),
            StrategyCounter::FINISHED { count: 9 }
        );
    }

    #[test]
    fn test_candidatehandlertype_basic_1_step() {
        // this is just a super basic test to review the mechanics.
        // this solves after one propagation
        logger_initiate(None);
        let config = ConfigType::new_extended(2, 2);

        let counts = vec![
            StrategyCounter::PENDING { count: 7 },
            StrategyCounter::PARTIALLY_FINISHED { count: 8 },
            StrategyCounter::FINISHED { count: 8 },
        ];
        // test_solver(&config, counts);
        let mut sht = StrategyHandlerType::new(&config);
        sht.test_solve(counts);
        // sht.solve();
        sht.verify();
    }

    #[test]
    fn test_candidatehandlertype_basic_2_steps() {
        logger_initiate(None);
        // this is just the basic testing without the propagation
        // this solves after two propagations
        let config = ConfigType::new_extended(3, 2);

        let counts = vec![
            StrategyCounter::PENDING { count: 17 },
            StrategyCounter::PARTIALLY_FINISHED { count: 21 }, // ATTENTION: THIS DECREASES COUNT!!!
            StrategyCounter::FINISHED { count: 21 },
        ];
        let mut sht = StrategyHandlerType::new(&config);
        sht.test_solve(counts);

        sht.verify();
    }

    #[test]
    fn test_candidatehandlertype_basic_3_steps() {
        // this is just the basic testing without the propagation
        // this solves after two propagations
        logger_initiate(None);
        let config = ConfigType::new_extended(2, 3);

        let counts = vec![
            StrategyCounter::PENDING { count: 15 },
            StrategyCounter::PARTIALLY_FINISHED { count: 18 },
            StrategyCounter::FINISHED { count: 18 },
        ];
        let mut sht = StrategyHandlerType::new(&config);
        sht.test_solve(counts);

        sht.verify();
    }
    // #[test]
    // fn test_candidatehandlertype_enhanced_test() {
    //     // this is just the basic testing without the propagation
    //     // this solves after two propagations
    //     logger_initiate(None);
    //     let config = ConfigType {
    //         colors: 5,
    //         columns: 3,
    //     };

    //     let mut sht = StrategyHandlerType::new(&config);
    //     sht.solve();

    //     sht.verify();
    // }
}
