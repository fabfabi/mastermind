/// module for basic functions around handling and grading the guesses
use rand::Rng;
use std::fmt;
// #[macro_use]
// extern crate fstrings;

///Structure to store the configuration, i.e. number of columns an colors
pub struct ConfigType {
    pub columns: usize,
    pub colors: u8,
}
impl ConfigType {
    pub fn done(&self) -> ResultType {
        ResultType {
            positions: self.columns as u8,
            colors: 0,
        }
    }
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
    pub fn is_finished(&self, configuration: &ConfigType) -> bool {
        configuration.columns == self.positions.into()
    }

    // check if the solution was found referencing to a number
    pub fn is_done_num(&self, width: usize) -> bool {
        width == self.positions.into()
    }

    fn string(&self) -> String {
        return f!("{p} {c}", p = &self.positions, c = &self.colors);
    }
}
impl fmt::Display for ResultType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "R{}{}", self.positions, self.colors)
    }
}
#[test]
fn test_result_type() {
    let r = ResultType::new(4, 1);
    println!("{}", r);
}
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
    pub fn new_check(entries: Vec<u8>, configuration: &ConfigType) -> Result<Self, CodeTypeError> {
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

    /// equals another CodeType defined as String (E.g. C1234)
    pub fn eq_str(&self, other: String) -> bool {
        return String::from(self) == other;
    }

    pub fn print(&self) {
        println!("{:?}", &self.entries)
    }

    ///grade with respect to another codeTpye
    pub fn grade(&self, other: &Self) -> ResultType {
        return grade(&self, &other);
    }

    /// return the length of the codeType for the case that a config cannot be accessed
    pub fn len(&self) -> usize {
        return self.entries.len();
    }

    /// deduce the "done" from the code since this contains the number of columns implicitly
    /// without the need for the configuration
    pub fn result_finished(&self) -> ResultType {
        ResultType {
            positions: self.len() as u8,
            colors: 0,
        }
    }
}
impl fmt::Display for CodeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let entries_as_string = &self
            .entries
            .iter()
            .map(|x| x.to_string())
            .collect::<String>();
        write!(f, "C{}", entries_as_string)
    }
}
impl From<&CodeType> for String {
    fn from(code: &CodeType) -> String {
        // not sure how to make that part work. Via the Display functionalities it does
        // but when trying to implement it to the string conversion, it does not...
        // let entries_as_string = code
        //     .entries
        //     .iter()
        //     .map(|x| x.to_string())
        //     .collect::<String>();
        return format!("{}", code);
    }
}
impl From<CodeType> for String {
    fn from(code: CodeType) -> String {
        // not sure how to make that part work. Via the Display functionalities it does
        // but when trying to implement it to the string conversion, it does not...
        // let entries_as_string = code
        //     .entries
        //     .iter()
        //     .map(|x| x.to_string())
        //     .collect::<String>();
        return format!("{}", code);
    }
}
#[test]
fn test_code_export() {
    let code = CodeType::new(vec![1, 2, 3, 4]);

    println!("some {}", code.clone());
    assert_eq!("C1234", format!("{}", code));

    let code_str: String = code.clone().into();
    assert_eq!("C1234", code_str);
}
#[test]
fn test_new_code_type_check() {
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

    println!("{}", CodeType::new(vec![1, 2, 3, 4]));
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
        raw_code.push(rand::rng().random_range(0..configuration.colors));
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
        return self.result.is_finished(&configuration);
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
