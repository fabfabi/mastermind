///module for io functions
use crate::mastermind::mastermind_mechanics as mm;
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
    let config = mm::ConfigType::new(4, 6);

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
