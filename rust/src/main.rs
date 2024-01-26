const COLUMNS: usize = 4;
const COLORS: u8 = 6;

#[derive(Copy, Clone)]
struct CodeType {
    entries : [u8; COLUMNS],
}
impl CodeType{
    fn grade(self : CodeType, solution : CodeType) -> ResultType{
        let mut line_bool = [false; COLUMNS];
        let mut solution_bool =  [false; COLUMNS];
        let mut positions : u8 = 0;
        let mut colors : u8 = 0;
    
        for i in 0..COLUMNS {
            if self.entries[i] == solution.entries[i]{
                positions += 1;
                line_bool[i] = true;
                solution_bool[i] = true;
            }
        }
    
        for i in 0..COLUMNS{
            if line_bool[i]{
                continue;
            }
            for j in 0..COLUMNS{
                if solution_bool[j]{
                    continue;
                }
                else if self.entries[i] == solution.entries[j] {
                    line_bool[i] = true;
                    solution_bool[j] = true;
                    colors += 1;
                    break
                }
            }
        }
        ResultType{positions, colors}
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct ResultType {
    positions : u8,
    colors : u8,
}

#[derive(Copy, Clone)]
struct LineType{
    code: CodeType,
    result: ResultType,
}
impl LineType {
    fn new(new_line : CodeType, solution : CodeType) -> LineType {
        let result = new_line.grade(solution);
        return LineType{code:new_line, result:result}
    }
}



fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_CodeType(){
        fn get_ct(a: u8, b: u8, c: u8, ref_number: u8) -> CodeType {
            let mut line= [ref_number; COLUMNS];
            line[0] = a;
            line[1] = b;
            line[2] = c;
            return CodeType{entries: line};
        }
        let a = get_ct(1,2,2,0);
        let b = get_ct(1,3,3,4);
        assert_eq!(b.grade(a), ResultType{positions:1,colors:0});
        let c = get_ct(9, 9, 9, 9);
        assert_eq!(c.grade(a), ResultType{positions: 0, colors: 0});
        let d = get_ct(2,1,3,4);
        let res = ResultType{positions:0, colors:2};
        assert_eq!(a.grade(d), res);
        let line = LineType::new(a, d);
        assert_eq!(line.result, res);
    }
    
}