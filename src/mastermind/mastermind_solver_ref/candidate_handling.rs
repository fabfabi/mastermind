/// old code to use as a reference.
/// The data structure is bad (global hashmap) some aspects might be re-used
/// Going for a directed graph (one node contains all its children)
use crate::mastermind::mastermind_mechanics::get_all_codes;
use crate::mastermind::mastermind_mechanics::grade;
use crate::mastermind::mastermind_mechanics::CodeType;
use crate::mastermind::mastermind_mechanics::ConfigType;
use crate::mastermind::mastermind_mechanics::ResultType;
use crate::mastermind::mastermind_solver::strategy_counter::count_and_clean;
use crate::mastermind::mastermind_solver::strategy_counter::StrategyCounter;
use crate::mastermind::mastermind_solver::strategy_counter::StrategyCounterTrait;
use std::collections::HashMap;
// use std::hash::Hash;

///class to handle the results of one candidate
#[derive(Clone)]
struct CandidateResultType {
    result_hashmap: HashMap<ResultType, Vec<CodeType>>,
    pub candidate: CodeType,
    //pub counter: StrategyCounter,
    number_of_candidates: usize,
}
impl CandidateResultType {
    fn new(guesses: &Vec<CodeType>, solution: &CodeType) -> Self {
        let mut map: HashMap<ResultType, Vec<CodeType>> = HashMap::new();

        for guess in guesses.iter() {
            let result = grade(&guess, &solution);
            map.entry(result)
                .or_insert_with(|| Vec::<CodeType>::new())
                .push(guess.clone()); //how would this work without the "clone"?
        }

        return CandidateResultType {
            result_hashmap: map,
            candidate: solution.clone(),
            //counter: StrategyCounter::Unfinished,
            number_of_candidates: guesses.len(),
        };
    }

    ///returns if a given result is present in that hashmap
    fn contains(&self, other_result: &ResultType) -> bool {
        let grade_map_keys: Vec<&ResultType> = self.result_hashmap.keys().collect();
        return grade_map_keys.contains(&other_result);
    }

    ///returns the overall number of different results
    fn num_results(&self) -> usize {
        return self.result_hashmap.keys().len();
    }

    ///returns the number of entries for a given result
    fn num_entries(&self, result: &ResultType) -> usize {
        if let Some(entries) = self.result_hashmap.get(result) {
            return entries.len();
        } else {
            return 0;
        }
    }

    /// show a high-level summary
    fn show(&self) {
        println!(
            "Candidate: '{}' with {} groups",
            self.candidate,
            self.result_hashmap.len()
        );
    }

    ///checks if two result_handler are equal (i.e. same results and same number of entries per resulg)
    fn eq(&self, other: &Self) -> bool {
        if !other.num_results() == self.num_results() {
            return false;
        }
        for (result, entries) in &self.result_hashmap {
            if entries.len() != other.num_entries(&result) {
                return false;
            }
        }

        return true;
    }

    ///return a vector of vectors of candidates
    fn get_candidate_lists(&self) -> Vec<Vec<CodeType>> {
        self.result_hashmap.values().map(|x| x.clone()).collect()
    }
}
impl StrategyCounterTrait for CandidateResultType {
    ///execute the counting logic
    fn count(&self, max: Option<usize>) -> StrategyCounter {
        if let Some(number) = max {
            //do not count if that does not make sense
            // best case scenario would be to get one right in the next step
            // and all others in the step after
            if number < 2 * self.number_of_candidates - 1 {
                //self.counter = StrategyCounter::Obsolete;
                return StrategyCounter::Obsolete;
            }
        }
        if self.num_results() == self.number_of_candidates {
            // if there is just one candidate left in this group
            // note: the ResultHandlerType ensures, that the last candidate is also taken
            if self.number_of_candidates == 1 {
                return StrategyCounter::Done { count: 1 };
            }

            let value: u16 = self.number_of_candidates as u16;

            // check if the solution was found and return the number of steps
            return match self.contains(&ResultType::new(self.candidate.len() as u8, 0)) {
                true => StrategyCounter::PartiallyFinished {
                    count: 2 * value - 1, // exact number is clear
                },
                _ => StrategyCounter::PartiallyFinished {
                    count: 2 * value, // worst case scenario, since the solution is not part of this step
                },
            };
        }
        // the best case how this could be solved
        return StrategyCounter::Unfinished {
            count: 2 * self.number_of_candidates as u16 - 1,
        };
    }
}
#[test]
fn test_result_handler() {
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

    assert!(matches!(
        result_handler.count(None),
        StrategyCounter::Unfinished { count: 9 }
    ));

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
    let result_handler_cnt1 = CandidateResultType::new(&guesses_cnt1, &solution);

    assert_eq!(
        result_handler_cnt1.count(None),
        StrategyCounter::Done { count: 1 }
    );

    //now testing also the count -> for two results
    let guesses_cnt2 = vec![
        CodeType::new(vec![1, 2, 3, 4]),
        CodeType::new(vec![1, 2, 3, 5]),
    ];
    let result_handler_cnt2 = CandidateResultType::new(&guesses_cnt2, &solution);
    assert_eq!(
        result_handler_cnt2.count(None),
        StrategyCounter::PartiallyFinished { count: 3 }
    );
    // check if the Obsolete path works
    assert_eq!(
        result_handler_cnt2.count(Some(2)),
        StrategyCounter::Obsolete // not better than the other one -> obsolete
    );
}

/// class to identify the next inputs to test
/// After entering a list of candidates (that are open), this class finds all possible next inputs
pub struct CandidateHandlerType {
    candidate_list: Vec<CandidateResultType>,
    count_storer: StrategyCounter,
}
impl CandidateHandlerType {
    pub fn new(candidates: &Vec<CodeType>, configuration: &ConfigType) -> Self {
        let mut result = CandidateHandlerType {
            candidate_list: Vec::new(),
            count_storer: StrategyCounter::Unfinished {
                count: 2 * candidates.len() as u16,
            },
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
        let all_candidates = get_all_codes(configuration);

        for code in all_candidates.iter() {
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

    /// check if a similar CandidateResult is already found
    fn contains_similar(&self, other_candidate_result: &CandidateResultType) -> bool {
        /* // if this input was already given, then all codes will have the same result
        if other_candidate_result.result_hashmap.len() == 1 {
            let candidate = other_candidate_result.candidate;

            return !() // check if there are more entries contained in this list
        } */

        for code in self.candidate_list.iter() {
            if code.eq(&other_candidate_result) {
                return true;
            }
        }

        return false;
    }

    /// return the number of candidates found
    fn len(&self) -> usize {
        return self.candidate_list.len();
    }

    /// check if a code is contained as a candidate
    fn contains(&self, other_candidate: &CodeType) -> bool {
        for candidate_result_type in self.candidate_list.iter() {
            if other_candidate.eq(&candidate_result_type.candidate) {
                return true;
            };
        }
        return false;
    }

    /// adds a candidate to the list
    fn add(&mut self, candidate: CandidateResultType) {
        self.candidate_list.push(candidate)
    }

    /// if this is the last entry, i.e. only one candidate left
    fn is_done(&self) -> bool {
        return self.candidate_list.len() == 1;
    }

    /// initiate the first search -> containing ALL combinations
    fn instantiate(configuration: &ConfigType) -> Self {
        Self::new(&get_all_codes(&configuration), &configuration)
    }

    /// update the count store and clean up the candidate list
    fn update_count_storer(&mut self, max: Option<usize>) {
        (self.candidate_list, self.count_storer) = count_and_clean(self.candidate_list.clone(), max)
    }

    ///identify next level candidates, add to the memory and return the next options as a hashmap
    fn create_next_candidates<'a>(
        &self,
        id_generator: &mut StepIDGenerator,
        memory: &mut HashMap<u128, StrategyStepType<'a>>,
        configuration: &'a ConfigType,
    ) -> HashMap<CodeType, std::ops::Range<u128>> {
        //use log::debug;
        let mut next_options = HashMap::new();
        //println!("adding {} candidates", &self.candidate_list.len());
        // for each individual candidate
        for candidate_result in &self.candidate_list {
            let id_start = id_generator.get_next();

            // get the individual lists of candidates that will be consumed
            let mut candidate_lists = candidate_result.get_candidate_lists();
            // println!("adding {} individual sub-results", candidate_lists.len());
            // add the id_range and the candidate to the StrategyStep
            let new_id_range = id_generator.get_range(candidate_lists.len() as u16);

            for (candidates, new_id) in candidate_lists.iter_mut().zip(new_id_range) {
                memory.insert(new_id, StrategyStepType::new(candidates, &configuration));
            }

            let id_end = id_generator.get_next(); //add one to include the highest in the range!
            next_options.insert(candidate_result.candidate.clone(), id_start..id_end);
        }
        return next_options;
    }

    fn show(&self) {
        println!(
            "CandidateHandler with {} candidates",
            self.candidate_list.len()
        );
    }
    fn show_details(&self) {
        self.show();
        for candidate in &self.candidate_list {
            candidate.show();
        }
    }
}
impl StrategyCounterTrait for CandidateHandlerType {
    fn count(&self, max: Option<usize>) -> StrategyCounter {
        // unpack the maximum
        if let Some(given_max_number) = max {
            //matcher to overrule to obsolete if a better strategy has been found
            let count_overruler = |t: StrategyCounter, num: usize| {
                if num >= given_max_number {
                    return StrategyCounter::Obsolete;
                }
                return t;
            };
            // and match to the current count
            match self.count_storer {
                StrategyCounter::Done { count: number } => {
                    return count_overruler(self.count_storer, number as usize)
                }
                StrategyCounter::PartiallyFinished { count: number } => {
                    return count_overruler(self.count_storer, number as usize)
                }
                StrategyCounter::Unfinished { count: number } => {
                    return count_overruler(self.count_storer, number as usize)
                }
                _ => return self.count_storer,
            }
        }
        // otherwise just return the count_storer
        return self.count_storer;
    }
}
/* //core::iter::traits::iterator;
impl Iterator for CandidateHandlerType {
    type Item = CandidateResultType;

    fn next(&mut self) -> Option<Self::Item> {
        return Some(self.candidate_list.last());
    }

} */

#[test]
fn test_candidatehandlertype() {
    let config = ConfigType {
        colors: 2,
        columns: 2,
    };
    let cht_one = CandidateHandlerType::new(&vec![CodeType::new(vec![0, 0])], &config);

    assert_eq!(cht_one.len(), 1);
    assert!(cht_one.is_done());

    let mut cht = CandidateHandlerType::new(
        &vec![
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

    cht.update_count_storer(None);

    assert_eq!(cht.count(None), StrategyCounter::Unfinished { count: 3 });

    //////////////////////////////////////////
    // now with only three as input
    let mut cht = CandidateHandlerType::new(
        &vec![
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

    cht.update_count_storer(None);
    assert_eq!(
        cht.count(None),
        StrategyCounter::PartiallyFinished { count: 5 } // 10 will calculate a partial finish at 5
    );

    //////////////////////////////////////////
    //  just a check on the high-level
    let config64 = ConfigType {
        colors: 6,
        columns: 4,
    };
    let cht = CandidateHandlerType::instantiate(&config64);
    assert_eq!(cht.len(), 5);
}
