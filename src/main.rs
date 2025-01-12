/// basic module for io functions
mod mastermind_io {

    ///Structure to store the configuration, i.e. number of columns an colors
    pub struct ConfigType {
        columns: usize,
        colors: u8,
    }

    ///Structure to store the result
    #[derive(PartialEq, Debug)]
    //#[derive(Copy, Clone, PartialEq, Debug)]
    struct ResultType {
        positions: u8,
        colors: u8,
    }

    impl ResultType {
        /// check if the solution was already found
        pub fn done(self, configuration: &ConfigType) -> bool {
            configuration.columns == self.positions.into()
        }
    }
    //#[derive(Copy, Clone)]
    ///code for one single try
    pub struct CodeType {
        entries: Vec<u8>, //[u8; COLUMNS],
                          //configuration: &'a ConfigType
    }
    impl CodeType {
        pub fn new(entries: Vec<u8>) -> Self {
            Self { entries }
        }
    }
    ///grade a code wrt a solution
    fn grade(guess: &CodeType, solution: &CodeType) -> ResultType {
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

    ///Line containing a code and the result
    //#[derive(Copy, Clone)]
    pub struct LineType {
        code: CodeType,
        result: ResultType,
    }
    impl LineType {
        fn new(new_line: CodeType, solution: &CodeType) -> LineType {
            let result = grade(&new_line, &solution);
            return LineType {
                code: new_line,
                result: result,
            };
        }

        ///was the solution found already?
        fn done(self, configuration: &ConfigType) -> bool {
            return self.result.done(&configuration);
        }
    }

    /* fn get_all_codes() -> Vec<CodeType> {
        let mut line = [0u8; COLUMNS];
        let mut all_possible_lines: Vec<CodeType> = Vec::new();
        all_possible_lines.push(CodeType { entries: line });
        //let num_combinations: u64 = u64::from(COLUMNS).pow(COLORS);

        fn augment(line: &mut [u8; COLUMNS], index: usize) -> Result<[u8; COLUMNS], &'static str> {
            if index == -1 {
                Err("All combinations found")
            } else if line[index] == (COLORS - 1) {
                line[index] = 0;
                augment(line, index - 1)
            } else {
                {
                    line[index] += 1
                }
            }
        }

        for i in 1..5 { //num_combinations{
        }

        return all_possible_lines;
    } */
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
        let b = LineType::new(b_raw, &a);
    }
}

fn main() {
    println!("Hello, world!");
}
