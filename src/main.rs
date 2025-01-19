/// basic module for io functions
mod mastermind_mechanics {

    use rand::Rng;

    ///Structure to store the configuration, i.e. number of columns an colors
    pub struct ConfigType {
        pub columns: usize,
        pub colors: u8,
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
    #[derive(Debug)]
    ///code for one single try
    pub struct CodeType {
        entries: Vec<u8>, //[u8; COLUMNS],
                          //configuration: &'a ConfigType
    }
    impl CodeType {
        pub fn new(entries: Vec<u8>) -> Self {
            Self { entries }
        }
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
        pub fn assert_eq(self, other_entries: Vec<u8>) -> bool {
            return self.entries == other_entries;
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

    use std::char::DecodeUtf16Error;
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

        /* println!("entries: {}", ac.len());
        for row in ac{
            println!("{:?}", row);
        }
        assert!(false) */
    }
}

mod mastermind_io {
    use crate::mastermind_mechanics as mm;
    use regex::Regex;
    //use std::io::{self, BufRead};
    //use std::num::ParseIntError;
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
        /*
        let re = Regex::new(r"[0-9]").unwrap();

        let results_str_vec: Vec<&str>;

        if let Ok(text) = result {
            results_str_vec = re.find_iter(text).map(|m| m.as_str()).collect();
        } else {
            results_str_vec = Vec::new();
            return Err(mm::CodeTypeError::DecodingError);
        } */
        /* let results_num: Vec<char>;

        if let Ok(text) = result {
            //: Vec<char> = input
            results_num = text
                .chars()
                .filter(|&c| "0123456789".contains(c))
                .map(|d| d as u8)
                .collect();
        } else {
            return Err(mm::CodeTypeError::DecodingError);
        } */
        let results_num: Vec<u8> = match result {
            Ok(text) => text
                .chars()
                .filter(|&c| "0123456789".contains(c)) // Filter digits
                .filter_map(|c| c.to_digit(10)) // Convert each character to a digit (u32)
                .map(|d| d as u8) // Convert u32 to u8
                .collect(), // Collect into a Vec<u8>
            Err(_) => return Err(mm::CodeTypeError::DecodingError), // Return an empty Vec if Result is Err
        };

        /*      let results_num: Vec<u8> = results_str_vec
        .iter()
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect(); */

        return mm::CodeType::new_check(results_num, &configuration);
    }

    fn test_read_code() {
        let config = mm::ConfigType {
            columns: 4,
            colors: 6,
        };

        assert!(read_code(Ok(String::new("1234")), &config)
            .unwrap()
            .assert_eq(vec![1, 2, 3, 4]))
    }

    fn _get_code(reader_function: fn() -> Result<mm::CodeType, mm::CodeTypeError>) -> mm::CodeType {
        loop {
            let resulting_row = reader_function();
            if let Ok(line) = resulting_row {
                return line;
            }
        }
    }
    /// read the user input and convert it to a valid CodeType
    pub fn get_code(configuration: &mm::ConfigType) -> mm::CodeType {
        /* fn reader_function() -> Result<mm::CodeType, mm::CodeTypeError> {
            return read_code(enter_code(), &configuration);
        } */

        loop {
            let resulting_row = read_code(enter_code(), &configuration);
            if let Ok(line) = resulting_row {
                return line;
            }
        }
    }

    #[test]
    fn test_enter_code() {}
}

mod testing {

    fn lifetime_check(variable: &i64) -> i64 {
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
}
fn main() {
    println!("Hello, world!");
}
