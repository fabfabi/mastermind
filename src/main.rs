use mastermind_mechanics::ConfigType;

#[macro_use]
extern crate fstrings;

/// module for basic functions around handling and grading the guesses
mod mastermind_mechanics {

    use rand::Rng;

    ///Structure to store the configuration, i.e. number of columns an colors
    pub struct ConfigType {
        pub columns: usize,
        pub colors: u8,
    }

    ///Structure to store the result
    #[derive(PartialEq, Debug, Copy, Clone, Hash, Eq)]
    pub struct ResultType {
        positions: u8,
        colors: u8,
    }

    impl ResultType {
        ///create a new element
        pub fn new(positions: u8, colors: u8) -> Self {
            return Self {
                positions: positions,
                colors: colors,
            };
        }
        /// check if the solution was already found
        pub fn done(&self, configuration: &ConfigType) -> bool {
            configuration.columns == self.positions.into()
        }
        fn string(&self) -> String {
            return f!("{p} {c}", p = &self.positions, c = &self.colors);
        }
    }
    #[derive(Clone, Debug)]
    ///code for one single try
    pub struct CodeType {
        entries: Vec<u8>, //[u8; COLUMNS],
                          //configuration: &'a ConfigType
    }
    impl CodeType {
        ///new WITHOUT any check
        pub fn new(entries: Vec<u8>) -> Self {
            Self { entries }
        }

        ///new with a check wrt to given config -> only needed for manual input stuff
        pub fn new_check(
            entries: Vec<u8>,
            configuration: &ConfigType,
        ) -> Result<Self, CodeTypeError> {
            if entries.len() != configuration.columns {
                return Err(CodeTypeError::ColumnMismatch);
            } else if *entries.iter().max_by_key(|x| *x).unwrap() >= configuration.colors {
                return Err(CodeTypeError::ColorMismatch);
            }
            return Ok(Self::new(entries));
        }
        ///equals a vector of other_entries
        pub fn eq_vec(&self, other_entries: Vec<u8>) -> bool {
            return self.entries == other_entries;
        }
        ///equals another CodeType
        pub fn eq(&self, other: &CodeType) -> bool {
            return self.eq_vec(other.entries.clone());
        }

        pub fn print(&self) {
            println!("{:?}", &self.entries)
        }

        ///grade with respect to another codeTpye
        pub fn grade(&self, other: &Self) -> ResultType {
            return grade(&self, &other);
        }
    }
    #[test]
    fn test_newCodeType_check() {
        let config = ConfigType {
            columns: 4,
            colors: 6,
        };

        assert_eq!(
            CodeType::new_check(vec![1, 2, 3, 4], &config)
                .unwrap()
                .entries,
            vec![1, 2, 3, 4]
        );

        assert_eq!(
            // this notation works. It could be refactored to get rid of the ::new method?! -> tbc if entries need to be pub
            CodeType {
                entries: vec![1, 2, 3, 4]
            }
            .entries,
            vec![1, 2, 3, 4]
        );

        // wrong number of columns should error
        assert!(CodeType::new_check(vec![1, 2, 3], &config).is_err());
        //wrong columns should error
        assert!(CodeType::new_check(vec![1, 2, 3, 6], &config).is_err());
        //no input
        assert!(CodeType::new_check(Vec::new(), &config).is_err());
    }
    ///grade a guess wrt a solution
    pub fn grade(guess: &CodeType, solution: &CodeType) -> ResultType {
        //const length: usize = 4; //&configuration.columns;
        let length: usize = guess.entries.len(); //&self.configuration.columns;
        if length != solution.entries.len() {
            panic!("Solution does not fit to entries")
        }

        //COLUMNS = self.config.columns;
        let mut line_bool = vec![false; length];
        let mut solution_bool = vec![false; length];
        let mut positions: u8 = 0;
        let mut colors: u8 = 0;

        for i in 0..length {
            if guess.entries[i] == solution.entries[i] {
                positions += 1;
                line_bool[i] = true;
                solution_bool[i] = true;
            }
        }

        for i in 0..length {
            if line_bool[i] {
                continue;
            }
            for j in 0..length {
                if solution_bool[j] {
                    continue;
                } else if guess.entries[i] == solution.entries[j] {
                    line_bool[i] = true;
                    solution_bool[j] = true;
                    colors += 1;
                    break;
                }
            }
        }
        ResultType { positions, colors }
    }

    /// Random code generator
    pub fn generate_code(configuration: &ConfigType) -> CodeType {
        let mut raw_code = Vec::new();
        for _ in 0..configuration.columns {
            raw_code.push(rand::thread_rng().gen_range(0..configuration.colors));
        }
        CodeType::new(raw_code)
    }

    ///Line containing a code and the result
    //#[derive(Copy, Clone)]
    pub struct LineType {
        code: CodeType,
        result: ResultType,
    }
    impl LineType {
        pub fn new(new_line: CodeType, solution: &CodeType) -> LineType {
            let result = grade(&new_line, &solution);
            return LineType {
                code: new_line,
                result: result,
            };
        }

        ///was the solution found already?
        pub fn done(&self, configuration: &ConfigType) -> bool {
            return self.result.done(&configuration);
        }

        pub fn print(&self) {
            println!("{:?} {:?}", &self.code, &self.result.string())
        }
    }

    pub fn get_all_codes(configuration: &ConfigType) -> Vec<CodeType> {
        let columns = configuration.columns;
        let colors = configuration.colors;

        let initial_line = vec![0u8; columns];
        let mut line = initial_line.clone();
        let mut all_possible_lines: Vec<CodeType> = Vec::new();
        let mut appender = |line: &Vec<u8>| {
            let mut new_line = line.clone();
            new_line.reverse();
            all_possible_lines.push(CodeType { entries: new_line });
        };
        appender(&line);
        loop {
            for c in line.iter_mut() {
                if *c == colors - 1 {
                    *c = 0;
                } else {
                    *c += 1;
                    break;
                }
            }
            if *line == vec![0u8; columns] {
                return all_possible_lines;
            }
            appender(&line)
        }
    }

    use std::error;
    use std::fmt;
    ///define custom Error Message
    #[derive(Debug, Clone)]
    pub enum CodeTypeError {
        ColumnMismatch,
        ColorMismatch,
        DecodingError,
    }
    impl fmt::Display for CodeTypeError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                Self::ColumnMismatch => {
                    write!(f, "Wrong columns defined")
                }
                Self::ColorMismatch => {
                    write!(f, "the provided the input has wrong colors")
                }
                Self::DecodingError => {
                    write!(f, "Error in decoding the input")
                }
            }
        }
    }
    impl error::Error for CodeTypeError {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            None
            /* match *self {
                CodeTypeError::ColumnMismatch => None,
                CodeTypeError::ColorMismatch => None,
            } */
        }
    }

    #[test]
    fn test_basics() {
        let a = CodeType::new(vec![1, 2, 2, 0]);
        let b_raw = CodeType::new(vec![1, 3, 3, 4]);
        let b = LineType::new(b_raw, &a);
        assert_eq!(
            b.result,
            ResultType {
                positions: 1,
                colors: 0
            }
        );

        let c_raw = CodeType::new(vec![9, 9, 9, 9]);
        let c = LineType::new(c_raw, &a);
        assert_eq!(
            c.result,
            ResultType {
                positions: 0,
                colors: 0
            }
        );
        let d_raw = CodeType::new(vec![2, 1, 3, 4]);
        let d = LineType::new(d_raw, &a);

        assert_eq!(
            d.result,
            ResultType {
                positions: 0,
                colors: 2,
            }
        );

        let configuration = ConfigType {
            colors: 6,
            columns: 4,
        };

        //d is not done
        assert!(!d.done(&configuration));

        let f_raw = CodeType::new(vec![1, 2, 2, 0]);
        let f = LineType::new(f_raw, &a);
        // f is done
        assert_eq!(
            f.result,
            ResultType {
                positions: 4,
                colors: 0,
            }
        );
        assert!(f.done(&configuration));
    }

    #[test]
    #[should_panic]
    fn test_wrong_input() {
        let a = CodeType::new(vec![1, 2, 2, 0]);
        let b_raw = CodeType::new(vec![1, 3, 3, 4, 5]);
        let _b = LineType::new(b_raw, &a);
    }

    #[test]
    fn test_all_combinations() {
        let config = ConfigType {
            columns: 4,
            colors: 6,
        };

        let ac = get_all_codes(&config);

        assert_eq!(ac.len(), usize::from(6_u16.pow(4)));

        let config = ConfigType {
            colors: 2,
            columns: 2,
        };
        let all_codes_small = get_all_codes(&config);
        let target_codes_small = vec![
            CodeType::new(vec![0, 0]),
            CodeType::new(vec![0, 1]),
            CodeType::new(vec![1, 0]),
            CodeType::new(vec![1, 1]),
        ];
        for (code, code_target) in all_codes_small.iter().zip(target_codes_small.iter()) {
            assert!(code.clone().eq(code_target));
        }

        /* println!("entries: {}", ac.len());
        for row in ac{
            println!("{:?}", row);
        }
        assert!(false) */
    }
}

///module for io functions
mod mastermind_io {
    use crate::mastermind_mechanics as mm;
    use text_io::read;

    fn enter_code() -> Result<String, text_io::Error> {
        let line: String = read!("{}\n");
        Ok(line)
    }

    ///convert a string format to a CodeType including error handling
    fn read_code(
        result: Result<String, text_io::Error>,
        configuration: &mm::ConfigType,
    ) -> Result<mm::CodeType, mm::CodeTypeError> {
        let results_num: Vec<u8> = match result {
            Ok(text) => text
                .chars()
                //.filter(|&c| "0123456789".contains(c)) // Filter digits -> not needed due tofilter_map
                .filter_map(|c| c.to_digit(10)) // Convert each character to a digit (u32)
                .map(|d| d as u8) // Convert u32 to u8
                .collect(), // Collect into a Vec<u8>
            Err(_) => return Err(mm::CodeTypeError::DecodingError), // Return an empty Vec if Result is Err
        };

        return mm::CodeType::new_check(results_num, &configuration);
    }

    #[test]
    fn test_read_code() {
        let config = mm::ConfigType {
            columns: 4,
            colors: 6,
        };

        // test the easy case
        assert!(read_code(Ok("1234".to_string()), &config)
            .unwrap()
            .eq_vec(vec![1, 2, 3, 4]));

        // test the other case
        assert!(read_code(Ok("12d f3, oiuf4f lksjf".to_string()), &config)
            .unwrap()
            .eq_vec(vec![1, 2, 3, 4]));

        //note: errors with incomplete columns/wrong colors are covered by CodeType::new_check

        let input: Result<String, text_io::Error> = Err(text_io::Error::MissingMatch);
        let result = read_code(input, &config);
        assert!(matches!(result, Err(mm::CodeTypeError::DecodingError)))
    }
    /// read the user input and convert it to a valid CodeType
    pub fn get_code(configuration: &mm::ConfigType) -> mm::CodeType {
        loop {
            let resulting_row = read_code(enter_code(), &configuration);
            if let Ok(line) = resulting_row {
                return line;
            }
        }
    }
}

mod mastermind_gameplay {
    use crate::mastermind_io;
    use crate::mastermind_mechanics;
    use crate::mastermind_mechanics::generate_code;
    use crate::mastermind_mechanics::CodeType;
    use crate::mastermind_mechanics::ConfigType;
    use crate::mastermind_mechanics::LineType;

    pub struct game<'a> {
        guesses: Vec<LineType>,
        solution: CodeType,
        configuration: &'a ConfigType,
    }

    impl<'a> game<'a> {
        pub fn new(configuration: &'a ConfigType) -> Self {
            let solution = generate_code(&configuration);
            solution.print();
            let mut gameplay = Self {
                guesses: Vec::<LineType>::new(),
                solution: solution,
                configuration: &configuration,
            };
            gameplay.play();
            return gameplay;
        }
        /// get new inputs until the solution was found
        fn play(&mut self) {
            loop {
                let done = self.guess();
                if done {
                    println!("!!!congratulations!!!");
                    break;
                }
                self.show()
            }
        }
        /// show all previous inputs
        fn show(&self) {
            for guess in &self.guesses {
                guess.print()
            }
        }
        ///read user input and store
        fn guess(&mut self) -> bool {
            let guess = LineType::new(mastermind_io::get_code(&self.configuration), &self.solution);
            let done = guess.done(&self.configuration);
            self.guesses.push(guess);

            return done;
        }
    }
}

mod mastermind_solver {
    use crate::mastermind_mechanics::get_all_codes;
    use crate::mastermind_mechanics::grade;
    use crate::mastermind_mechanics::CodeType;
    use crate::mastermind_mechanics::ConfigType;
    use crate::mastermind_mechanics::ResultType;
    use std::collections::HashMap;

    ///class to handle the results of one candidate
    struct CandidateResultType {
        result_hashmap: HashMap<ResultType, Vec<CodeType>>,
        pub candidate: CodeType,
    }
    impl CandidateResultType {
        pub fn new(guesses: &Vec<CodeType>, solution: &CodeType) -> Self {
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
            };
        }

        ///returns if a given result is present in that hashmap
        pub fn contains(&self, other_result: &ResultType) -> bool {
            let grade_map_keys: Vec<&ResultType> = self.result_hashmap.keys().collect();
            return grade_map_keys.contains(&other_result);
        }

        ///returns the overall number of different results
        pub fn num_results(&self) -> usize {
            return self.result_hashmap.keys().len();
        }

        ///returns the number of entries for a given result
        pub fn num_entries(&self, result: &ResultType) -> usize {
            if let Some(entries) = self.result_hashmap.get(result) {
                return entries.len();
            } else {
                return 0;
            }
        }

        ///checks if two result_handler are equal (i.e. same results and same number of entries per resulg)
        pub fn eq(&self, other: &Self) -> bool {
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

        let guesses_neq = vec![
            CodeType::new(vec![1, 2, 3, 4]),
            CodeType::new(vec![2, 2, 2, 2]),
            CodeType::new(vec![3, 3, 3, 3]),
            CodeType::new(vec![4, 4, 4, 4]),
        ];
        let result_handler_neq = CandidateResultType::new(&guesses_neq, &solution);
        assert!(!result_handler.eq(&result_handler_neq));

        let guesses_eq = vec![
            CodeType::new(vec![1, 2, 3, 4]),
            CodeType::new(vec![2, 2, 2, 2]),
            CodeType::new(vec![3, 3, 3, 3]),
            CodeType::new(vec![4, 4, 4, 4]),
            CodeType::new(vec![0, 2, 0, 0]),
        ];
        let result_handler_eq = CandidateResultType::new(&guesses_eq, &solution);
        assert!(result_handler.eq(&result_handler_eq));
        assert!(result_handler.candidate.eq(&solution))
    }

    /// class to identify the next inputs to test
    struct CandidateHandlerType {
        candidate_list: Vec<CandidateResultType>,
    }
    impl CandidateHandlerType {
        pub fn new(candidates: Vec<CodeType>, configuration: &ConfigType) -> Self {
            let mut result = CandidateHandlerType {
                candidate_list: Vec::new(),
            };
            // if there is only one candidate left -> no more grading needed
            if candidates.len() == 1 {
                let candidate = candidates[0].clone();
                let new_candidate_result =
                    CandidateResultType::new(&vec![candidate.clone()], &candidate);
                result.add(new_candidate_result);
                return result;
            }
            let all_candidates = get_all_codes(configuration);

            for code in all_candidates.iter() {
                let new_candidate_result = CandidateResultType::new(&candidates, code);
                if !result.contains_similar(&new_candidate_result) {
                    result.add(new_candidate_result)
                }
            }

            return result;
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

        /// return the number of candidates found
        pub fn len(&self) -> usize {
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
    }
    #[test]
    fn test_CandidateHandlerType() {
        let config = ConfigType {
            colors: 2,
            columns: 2,
        };
        let cht_one = CandidateHandlerType::new(vec![CodeType::new(vec![0, 0])], &config);

        assert_eq!(cht_one.len(), 1);

        let cht = CandidateHandlerType::new(
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
    }
}

mod testing {
    use std::result::Iter;

    use crate::mastermind_mechanics::grade;
    use crate::mastermind_mechanics::CodeType;
    use crate::mastermind_mechanics::ConfigType;
    use crate::mastermind_mechanics::ResultType;

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
fn main() {
    println!("Hello, world!");
    use mastermind_gameplay::game as mastermind;
    use mastermind_mechanics::ConfigType;
    let configuration = ConfigType {
        columns: 4,
        colors: 6,
    };
    let mut _game = mastermind::new(&configuration);
}
