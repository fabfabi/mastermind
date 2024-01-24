const COLUMNS: usize = 4;
const COLORS: u8 = 6;

struct CodeType {
    entries : [u8; COLUMNS],
}
impl CodeType{
    fn copy(self) -> [u8; COLUMNS]{
        return self.entries
    }
    fn grade(self : CodeType, solution_in : CodeType) -> ResultType{
        let mut line = self.copy();
        let mut solution = solution_in.copy();
        let mut positions : u8 = 0;
        let mut colors : u8 = 0;
    
        for i in 0..COLUMNS {
            if line[i] == solution[i]{
                positions += 1;
                line[i] = 0;
                solution[i] = 0;
            }
        }
    
        for i in 0..COLUMNS{
            if line[i] == 0{
                continue;
            }
            for j in 0..COLUMNS{
                if solution[j] == 0{
                    continue;
                }
                else if line[i] == solution[j] {
                    line[i] = 0;
                    solution[j] = 0;
                    colors += 1;
                    break
                }
            }
        }
        ResultType{positions, colors}
    }
}

struct ResultType {
    positions : u8,
    colors : u8,
}

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
