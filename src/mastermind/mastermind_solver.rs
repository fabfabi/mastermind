mod mastermind_solver {
    use crate::mastermind::mastermind_mechanics::get_all_codes;
    use crate::mastermind::mastermind_mechanics::grade;
    use crate::mastermind::mastermind_mechanics::CodeType;
    use crate::mastermind::mastermind_mechanics::ConfigType;
    use crate::mastermind::mastermind_mechanics::ResultType;
    use std::collections::HashMap;
    use std::hash::Hash;

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
    struct CandidateHandlerType {
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
            (self.candidate_list, self.count_storer) =
                count_and_clean(self.candidate_list.clone(), max)
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

    ///trait designed to support the counting function.
    /// If all are done -> done
    /// if at least one is done or partially finished -> partially finished
    /// if all are obsolete -> obsolete
    /// otherwise return Unfinished
    ///
    /// Important:
    ///  * count needs to return: if count < max => Obsolete
    trait StrategyCounterTrait {
        fn count(self: &Self, max: Option<usize>) -> StrategyCounter;
    }

    /// function to do the counting and cleaning up of all obsolete paths
    fn count_and_clean<T>(
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
    enum StrategyCounter {
        Start, // for the first node
        //Option,                           // several options for how to continue
        Unfinished { count: u16 },        // counting not yet done
        PartiallyFinished { count: u16 }, //first count available
        Done { count: u16 },              // this strategy path finished already
        Obsolete,                         // This path has more moves than a known path
                                          //End,                              // this is the last node of the strategy
    }

    ///simple struct to generate IDs for every new step
    struct StepIDGenerator {
        max_id: u128,
    }
    impl StepIDGenerator {
        fn new() -> Self {
            StepIDGenerator { max_id: 0 }
        }
        /// return a range of 'n' IDs
        fn get_range(&mut self, n: u16) -> std::ops::Range<u128> {
            let number = self.max_id.clone();
            self.max_id += n as u128;
            return number..self.max_id;
        }

        ///get the highest number used
        fn get_highest(&self) -> u128 {
            // this could create a negative number. Do I need to pay attention for that?
            return self.max_id - 1;
        }

        ///get the next id that will be used
        fn get_next(&self) -> u128 {
            return self.max_id;
        }
    }
    #[test]
    fn test_idgenerator() {
        let mut generator = StepIDGenerator::new();
        assert_eq!(generator.get_range(5), 0..5);
        assert_eq!(generator.get_highest(), 4);
        assert_eq!(generator.get_next(), 5);
        assert_eq!(generator.get_range(5), 5..10);
        assert_eq!(generator.get_next(), 10);
    }
    /// enum to contain either a list of candidates or a candidate handler
    /// this is needed in order to seprate the creation of the strategy step type from the
    /// iteration step.
    /// This allows parallelization by removing the borrow once it is initiated
    enum CandidateOption {
        empty,
        raw { candidates: Vec<CodeType> },
        instantiated { handler: CandidateHandlerType },
    }
    impl CandidateOption {
        /// return MUTABLE reference to the CandidateHandler
        fn mutate(&mut self) -> &mut CandidateHandlerType {
            if let CandidateOption::instantiated { handler } = self {
                return handler;
            } else {
                panic!("CandidateOption is not yet instantiated")
            }
        }
        /// return immutable reference to the CandidateHandler
        fn borrow<'a, 'b>(&'a self) -> &'b CandidateHandlerType
        where
            'a: 'b,
        {
            if let CandidateOption::instantiated { handler } = self {
                return handler;
            } else {
                panic!("CandidateOption is not yet instantiated")
            }
        }

        /// instantiate to replace the borrowed reference with the Candidate Handler
        fn instantiate<'a>(&mut self, configuration: &'a ConfigType) {
            //self.show();
            if let CandidateOption::raw { candidates } = self {
                *self = CandidateOption::instantiated {
                    handler: CandidateHandlerType::new(candidates, configuration),
                };
            } else {
                println!("CandidateOption is already instantiated")
            }
        }

        fn show(&self) {
            match &self {
                CandidateOption::instantiated { handler } => handler.show(),
                CandidateOption::empty => println!("Empty Candidate Opion"),
                CandidateOption::raw { candidates } => {
                    println!("Raw Candidate Option with {} candidates", candidates.len())
                }
                _ => println!("unknown status"),
            }
        }
    }

    /// Structure to handle the strategy.
    /// Main Idea for every step:
    ///   * StrategyType handles is responsible for the high-level handling (i.e. creation + iterations)
    ///   * The StrategyStepType types handles all different possible options (i.e. identifying how to break-down + counting)
    ///   * it wraps around the CandidateHandlerType
    /// Structure to handle different options for the Strategy
    /// i.e. one of these steps could be next
    /// It links to several StrategyStepTypes

    struct StrategyStepType<'b> {
        counter: StrategyCounter,
        candidate_option: CandidateOption,
        //next_options: Vec<u128>,
        next_options: HashMap<CodeType, std::ops::Range<u128>>,
        configuration: &'b ConfigType,
    }
    impl StrategyStepType<'_> {
        ///initiate everything -> very first step
        fn initiate_beginning<'b, 'c>(configuration: &'b ConfigType) -> StrategyStepType<'c>
        where
            //'a: 'c,
            'b: 'c,
        {
            let candidate_handler = CandidateHandlerType::instantiate(&configuration);
            let length = candidate_handler.candidate_list.len() as u16;
            return StrategyStepType {
                counter: StrategyCounter::Unfinished { count: 2 * length },
                candidate_option: CandidateOption::instantiated {
                    handler: candidate_handler,
                },
                next_options: HashMap::new(),
                configuration: &configuration,
            };
        }
        /// create a new Strategy Step type in the control flow based on a borrowed vector of CodeTypes
        fn new_ref<'b, 'c>(
            candidates: Vec<CodeType>,
            configuration: &'b ConfigType,
        ) -> StrategyStepType<'c>
        where
            'b: 'c,
        {
            let length = candidates.len() as u16;
            return StrategyStepType {
                counter: StrategyCounter::Unfinished { count: 2 * length },
                candidate_option: CandidateOption::raw {
                    candidates: candidates,
                },
                next_options: HashMap::new(),
                configuration: &configuration,
            };
        }
        /// create a new Strategy Step type in the control flow
        fn new<'a, 'b, 'c>(
            candidates: &'a Vec<CodeType>,
            configuration: &'b ConfigType,
        ) -> StrategyStepType<'c>
        where
            'b: 'c,
        {
            let mut sst = Self::new_ref(candidates.clone(), &configuration);
            sst.instantiate();
            return sst;
        }

        /// convert all the candidate options from "raw" to "initiated"
        /// this costs some calculation time since all possible candidates are checked
        fn instantiate(&mut self) {
            self.candidate_option.instantiate(self.configuration);
        }

        /// add the ids as next options
        fn add_ids(&mut self, candidate: &CodeType, id_range: std::ops::Range<u128>) {
            //let new_items = id_range.collect();
            //let id_vector = Vec::from_iter(id_range);
            self.next_options
                .insert(candidate.clone(), id_range.clone());
        }
        /// work with the candidate handler as mutable reference
        fn mutate(&mut self) -> &mut CandidateHandlerType {
            return self.candidate_option.mutate();
        }

        /// borrow the candidate handler
        fn borrow<'a>(&'a self) -> &'a CandidateHandlerType {
            return self.candidate_option.borrow();
        }

        /// create the next candidates
        fn create_next_candidates<'a, 'b>(
            &mut self,
            memory: &mut HashMap<u128, StrategyStepType<'a>>,
            id_generator: &mut StepIDGenerator,
            configuration: &'b ConfigType, // just for the lifetime
        ) where
            'b: 'a,
        {
            if let CandidateOption::instantiated { handler } = &mut self.candidate_option {
                self.next_options =
                    handler.create_next_candidates(id_generator, memory, configuration);
            } else {
                panic!("not yet instantiated")
            }
        }

        /// update the counter
        fn update_counter(&mut self) {
            match &self.candidate_option {
                CandidateOption::instantiated { handler } => {
                    self.counter = handler.count_storer.clone()
                }
                _ => panic!("Candidate not yet instantiated!"),
            }
        }

        fn show(&self) {
            self.candidate_option.show()
        }

        // return a vector containing all children including their children
        fn get_all_children(&self, memory: &HashMap<u128, StrategyStepType>) -> Vec<u128> {
            if self.next_options.is_empty() {
                return vec![];
            }

            // convert a hashmap with ranges to a flat vector of numbers
            let convert_to_vec = |v: &HashMap<CodeType, std::ops::Range<u128>>| {
                v.values().cloned().flatten().collect::<Vec<_>>()
            };
            let mut children: Vec<u128> = convert_to_vec(&self.next_options);

            let mut childrens_children: Vec<u128> = children
                .clone()
                .into_iter()
                // flat_map does the recursive things
                .flat_map(|v| match memory.get(&v) {
                    Some(strategy_step) => strategy_step.get_all_children(&memory),
                    _ => vec![],
                })
                .collect();
            children.append(&mut childrens_children);

            return children;
        }
    }

    #[test]
    fn test_strategysteptype() {
        let configuration = ConfigType {
            colors: 2,
            columns: 2,
        };

        //let sst_one = StrategyStepType::new(&vec![CodeType::new(vec![0, 1])], &config);
        //assert_eq!(sst_one.counter, StrategyCounter::Unfinished);

        let mut sst = StrategyStepType::initiate_beginning(&configuration);

        sst.mutate().update_count_storer(None);
        assert_eq!(
            sst.borrow().count_storer,
            StrategyCounter::Unfinished { count: 3 },
        );
        let mut memory: HashMap<u128, StrategyStepType> = HashMap::new();

        assert_eq!(sst.get_all_children(&memory), vec![]);

        let mut id_generator = StepIDGenerator::new();

        sst.create_next_candidates(&mut memory, &mut id_generator, &configuration);
        let mut all_children = sst.get_all_children(&memory).clone();
        all_children.sort();
        assert_eq!(all_children, vec![0, 1, 2, 3, 4, 5]);
    }

    /// high level object to handle the strategy.
    /// It manages all the individual step types as well as the connections
    /// All individual steps are stored in one giant hashmap
    struct StrategyHandler<'a, 'b>
    where
        'b: 'a,
    {
        id_generator: StepIDGenerator,
        memory: HashMap<u128, StrategyStepType<'a>>,
        configuration: &'b ConfigType,
        max_level: u8,
        current_level: u8,
        level_keys: Vec<(u128, u128)>,
    }
    impl StrategyHandler<'_, '_> {
        pub fn new<'a>(configuration: &'a ConfigType, max_level: u8) -> StrategyHandler<'a, 'a> {
            let mut hashmap: HashMap<u128, StrategyStepType> = HashMap::new();

            let step_handler = StrategyStepType::initiate_beginning(&configuration);
            //retrieve the initial value as 0
            let mut id_generator = StepIDGenerator::new();
            id_generator.get_range(1);
            hashmap.insert(0 as u128, step_handler);

            let strategy_handler = StrategyHandler {
                id_generator: id_generator,
                configuration: &configuration,
                memory: hashmap,
                max_level: max_level,
                current_level: 0,
                level_keys: vec![(0, 1)],
            };

            return strategy_handler;
        }

        /// recursively removes all children of one branch
        fn cut_branch(&mut self, branch_id: u128) {
            let branch = match self.memory.remove(&branch_id) {
                Some(branch) => branch,
                _ => return,
            };

            let all_child_steps = branch.get_all_children(&self.memory);

            for child_id in all_child_steps {
                self.cut_branch(child_id);
            }
        }

        /// runs the full algorithm
        /// verifies completeness by having only one candidate in the highest level in status "done"
        fn solve(&mut self) {}

        /// first tep of the propagation where the next level is created
        /// TODO Enhancement for the future: lazy evaluation
        fn create_next_level(&mut self) {
            // store the currently highest number
            let level_begin_next = self.id_generator.get_highest() + 1;

            // get the keys from the last level. This will be propagated
            let (level_begin_current, level_end_current) =
                if let Some(&tuple) = self.level_keys.last() {
                    tuple
                } else {
                    panic!("no level boundaries")
                };
            //////////////////////////////////////////////////////////////////////////////////////
            // 1. create the next level
            for sst_id in level_begin_current..=level_end_current {
                //remove the value and insert it in the end to avoid two mutable borrows at the same time
                let mut handler = self.memory.remove(&sst_id).unwrap();
                handler.create_next_candidates(
                    &mut self.memory,
                    &mut self.id_generator,
                    &self.configuration,
                );
                //and insert
                self.memory.insert(sst_id, handler);
            }

            // and save the boundaries of the last level
            let level_end_next = self.id_generator.get_highest();
            self.level_keys.push((level_begin_next, level_end_next));
        }

        /// 3. calculate the count for all candidates
        fn calculate_count(&mut self) {
            for (focus_lvl_begin, focus_lvl_end) in self.level_keys.iter().rev() {
                for sst_id in *focus_lvl_begin..=*focus_lvl_end {
                    //remove the value and insert it in the end to avoid two mutable borrows at the same time
                    let handler_option = self.memory.remove(&sst_id);
                    let mut handler = match handler_option {
                        None => continue,
                        Some(handler) => handler,
                    };
                    handler.update_counter();

                    //and insert again
                    self.memory.insert(sst_id, handler);
                }
            }
        }

        /// 4. delete obsolete steps
        fn delete_obsolete(&mut self) {
            // clone keys to avoid immutable borrow in order to be able to cut branches
            let level_keys = self.level_keys.clone();

            for (focus_lvl_begin, focus_lvl_end) in level_keys.iter() {
                for sst_id in *focus_lvl_begin..=*focus_lvl_end {
                    // remove the value and insert it in the end to avoid two mutable borrows at the same time
                    let handler_option = self.memory.remove(&sst_id);
                    let handler = match handler_option {
                        Some(handler) => handler,
                        None => continue,
                    };

                    // check if it is obsolete
                    let cut_branch_bool = match handler.counter {
                        StrategyCounter::Obsolete => true,
                        _ => false,
                    };

                    // and insert again
                    self.memory.insert(sst_id, handler);

                    if cut_branch_bool {
                        self.cut_branch(sst_id);
                    }
                }
            }
        }

        /// create the next level
        /// 1. create the next level (serial)
        /// 2. instantiate the next level (i.e. identify the best candidates) -> heavy lifting!!!
        /// 3. calculate the count (backwards from the last level to the highets one)
        /// 4. forward pass to delete all steps that are obsolete
        fn propagate(&mut self) {
            if self.current_level > self.max_level {
                println!("last level reached");
                panic!("the solution was not found until last level -> fix");
            }
            // chop it into several functions in order to test it individually
            //////////////////////////////////////////////////////////////////////////////////////
            // 1. Create the next level
            self.create_next_level();

            // Note: instantiating is not needed since step 1 instantiated directly
            // enhancement for the future to use two steps for this
            // //////////////////////////////////////////////////////////////////////////////////////
            // // 2. instantiate the next level -> Heavy lifting!!! TODO -> FEARLESS CONCURRENCY
            // for sst_id in new_lvl_begin..=new_lvl_end {
            //     //remove the value and insert it in the end to avoid two mutable borrows at the same time
            //     let mut handler = self.memory.remove(&sst_id).unwrap();
            //     handler.instantiate();
            //     //and insert again
            //     self.memory.insert(sst_id, handler);
            // }

            //////////////////////////////////////////////////////////////////////////////////////
            // 3. calculate the count (backwards from the last level to the highets one)
            self.calculate_count();

            //////////////////////////////////////////////////////////////////////////////////////
            // 4. forward pass to delete all steps that are obsolete
            self.delete_obsolete();
        }
    }

    #[test]
    fn test_strategyhandler() {
        let configuration = ConfigType {
            colors: 3,
            columns: 3,
        };

        //let sst_one = StrategyStepType::new(&vec![CodeType::new(vec![0, 1])], &config);
        //assert_eq!(sst_one.counter, StrategyCounter::Unfinished);

        let mut sht = StrategyHandler::new(&configuration, 8);

        sht.create_next_level(); // -> does not make sense yet

        //assert!(false)
    }
}
mod testing {
    use std::collections::HashMap;
    use std::result::Iter;

    use crate::mastermind::mastermind_mechanics::grade;
    use crate::mastermind::mastermind_mechanics::CodeType;
    use crate::mastermind::mastermind_mechanics::ConfigType;
    use crate::mastermind::mastermind_mechanics::ResultType;

    fn lifetime_check(_: &i64) -> i64 {
        let return_value: i64 = 50;

        return return_value;
    }
    #[test]
    fn test_lifetime_check() {
        assert_eq!(lifetime_check(&10), 50)
    }
    #[test]
    fn test_vectors() {
        let string_vec = vec!["1", "2", "3"];
        let translation: Vec<u8> = string_vec
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(translation, vec![1, 2, 3])
    }

    #[test]
    fn test_iterator() {
        use rayon::prelude::*;
        // test an iterator that might have a variable length
        struct iter_test_class {
            value_list: Vec<i32>,
        }
        impl iter_test_class {
            pub fn new(values: Vec<i32>) -> Self {
                iter_test_class { value_list: values }
            }
            pub fn add(&mut self, val: i32) -> Option<i32> {
                self.value_list.push(val.clone() * 2);
                return Some(val);
            }
        }
        impl Iterator for iter_test_class {
            type Item = i32;
            fn next(&mut self) -> Option<Self::Item> {
                //let val = self.v.pop()?; // "?" unpack the result and if it fails, return the error

                // return the result from the match statement
                match self.value_list.pop() {
                    Some(val) if val < 10 => self.add(val),
                    Some(val) => Some(val),
                    _ => None,
                }
            }
        }
        let object = iter_test_class::new(vec![1, 2, 3]);

        for val in object {
            println!("{val}")
        }
        /* let object2 = iter_test_class::new(vec![1, 2, 3]);
        let result: Vec<_> = object2
            .par_iter()
            .map(|&x| x * 2) // Multiply each element by 2
            .collect(); */
    }
}
