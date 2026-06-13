// use crate::mastermind::mastermind_mechanics::get_all_codes;
// use crate::mastermind::mastermind_mechanics::grade;
use crate::mastermind::mastermind_mechanics::CodeType;
use crate::mastermind::mastermind_mechanics::ConfigType;
// use crate::mastermind::mastermind_mechanics::ResultType;
// use crate::mastermind::mastermind_solver::strategy_counter::count_and_clean;
use crate::mastermind::mastermind_solver::strategy_counter::StrategyCounter;
// use crate::mastermind::mastermind_solver::strategy_counter::StrategyCounterTrait;
use crate::mastermind::mastermind_solver::candidate_handling::CandidateHandlerType;
use std::collections::HashMap;

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
            self.next_options = handler.create_next_candidates(id_generator, memory, configuration);
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
        let (level_begin_current, level_end_current) = if let Some(&tuple) = self.level_keys.last()
        {
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
