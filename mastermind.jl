
"""Basic class to represent the internal functions needed for mastermind.
Import it via "using .MASTERMIND" (after include("mastermind.jl")).
play() lets you play the game.
"""
module MASTERMIND
    using Random
    using CSV
    using Distributed
    using SharedArrays
    using Test
    using Logging
    using DataStructures
    using ParallelUtilities

    import Base.== #for overloading the operator

    export COLORS, COLUMNS, RESULT, CANDIDATES, LINE, SOLUTION

    export RAW_LINE, LINE, RESULT

    export grade_result, auto_solver, result_generator, read_results, play, raw_line_list

    ############################################################################
    #####################    GLOBAL VARIABLES   ################################
    ############################################################################

    """Number of possible colors"""
    COLORS  = 6
    """Number of columns"""
    COLUMNS = 4


    ############################################################################
    ###############     CODE           #########################################
    ############################################################################


    random_result() = RAW_LINE(vec(rand(1 : COLORS, (1,COLUMNS))))

    """the result of the grading.
    pos are the correct positions
    cols are the correct colors"""
    struct RESULT
        pos  :: Int8
        cols :: Int8
    end
    # overload the == operator
    Base.:(==)(c::RESULT, d::RESULT) = ((c.pos == d.pos) & (c.cols == d.cols)) 
    Base.:(!=)(c::RESULT, d::RESULT) = (!(c == d)) 
    Base.:(==)(c::RESULT, d::Bool)   = (d ? c.pos == COLUMNS : c.pos < COLUMNS) #the correct result is true
    Base.:(!=)(c::RESULT, d::Bool)   = (!(c == d)) #the correct result is true

    @test RESULT(2,1) == RESULT(2,1)
    @test RESULT(2,1) != RESULT(1,1)
    @test (RESULT(COLUMNS,0) == true)
    @test (RESULT(COLUMNS-1,0) != true)

    """resembles one raw line of input (excl. result)"""
    struct RAW_LINE
        code :: Vector{Int8}
    end

    Base.:(==)(a::RAW_LINE, b::RAW_LINE) = begin
        if size(a.code) != size(b.code)
            return false
        end
        rows = size(a.code)[1]
        for i = 1:rows
            if a.code[i] != b.code[i]
                return false
            end
        end
        return true

    end
    Base.:(!=)(a::RAW_LINE, b::RAW_LINE) = (!(a == b))

    @test RAW_LINE([1, 2, 3]) == RAW_LINE([1, 2, 3])
    @test RAW_LINE([1, 2, 3]) != RAW_LINE([1, 2, 4])
    @test RAW_LINE([1, 2, 3]) != RAW_LINE([1, 2, 3, 4])

    """resembles one line of input plus the graded result"""
    struct LINE
        code :: RAW_LINE
        result :: RESULT
    end
    Base.:(==)(a::LINE, b::LINE) = ((a.code == b.code) & (a.result == b.result))
    Base.:(!=)(a::LINE, b::LINE) = (!(a == b))

    mutable struct SOLUTION
        input :: RAW_LINE
        next_step :: Dict{RESULT, SOLUTION}
        candidates :: Array{RAW_LINE}
    end

    #=
    println(c :: RAW_LINE) = Base.println(c.code)
    println(l :: LINE) = Base.println(l.code.code)
    println(x) = Base.println(x)
    =#

    function show(line :: LINE)
        mem = ""
        for i in line.code.code #1:size(line.code)[1]
            mem *= string(i) #line.code[1, i])
            mem *= " "
        end
    
        mem *= " "
        mem *= " "
        mem *= string(line.result.pos)
        mem *= " "
        mem *= string(line.result.cols)
        #mem *= " \n"
        println(mem)
    end
    

    
    """grades how good the input matches the solution. This function is symmetric.
    It returns the type RESULT"""
    function grade_result(input::RAW_LINE, solution::RAW_LINE)
        

        in = copy(input.code)
        sol = copy(solution.code)

        correct_positions = 0
        correct_colors = 0


        for i = 1: COLUMNS
            if in[i] == sol[i]
                correct_positions += 1
                in[i] = 0
                sol[i] = 0
            end
        end

        if correct_positions == COLUMNS
            return RESULT(COLUMNS, 0)
        end

        for i = 1: COLUMNS, j = 1: COLUMNS
            if i == j
                continue
            elseif in[i] == 0 || sol[j] == 0
                continue
            elseif in[i] == sol[j]
                correct_colors += 1
                in[i] = 0
                sol[j] = 0
            end
        end
        #return correct_positions, correct_colors
        return RESULT(correct_positions, correct_colors)
    end
    gradetest(a, b) = grade_result(RAW_LINE(vec(a)), RAW_LINE(vec(b)))
    @test gradetest([1, 3, 3, 5], [2, 3, 3, 6]) == RESULT(2, 0)
    @test gradetest([1, 3, 3, 5], [1, 3, 3, 6]) == RESULT(3, 0)
    @test gradetest([1, 2, 3, 5], [2, 3, 1, 6]) == RESULT(0, 3)
    @test gradetest([1, 2, 4, 5], [1, 4, 3, 6]) == RESULT(1, 1)

    function full_list()

        rows = COLORS ^ COLUMNS
        data = Matrix(undef, rows, COLUMNS)
        

        function add_val(ref)
            new = copy(ref)
            for col = COLUMNS:-1:1
                if new[col] == COLORS
                    new[col] = 1 #Reset to the lowest value, the next one will be increased
                else
                    new[col] += 1 #augment
                    break
                end
            end
            return new
        end

        data[1,:] = [1 for i = 1:COLUMNS]'
        for r = 2:rows
            data[r, :] = add_val(data[r-1, :])
        end

        return data

    end

    function raw_line_list()
        liste = full_list()
        rows, _ = size(liste)
        output = RAW_LINE[]
        for i = 1:rows
            push!(output, RAW_LINE(vec(liste[i, :])))
        end

        return output
    end


    function array_list()
        liste = full_list()
        rows, _ = size(liste)
        output = Array{Any}[]

        for i = 1:rows
            push!(output, liste[i, :])
        end

        return output
    end

    """List of all possible combinations"""
    CANDIDATES = array_list()
    
    """automatically finds the solution by using random values and checking wrt previous results
    """
    function auto_solver(solution)
        
        candidates = copy(CANDIDATES)

        #candidates = full_list() #does not work with the filter function
        choices = Any[]
        results = Any[]

        selected_row = 0 # to get it into this namespace
        result = false

        while result != true #RESULT is true iff it is correct
            rows = size(candidates)[1]
            selected_row = candidates[rand(1:rows)]

            result = grade_result(selected_row, solution)

            push!(choices, selected_row)
            push!(results, result)

            filter!(x -> grade_result(x, selected_row) == result, candidates)
        
        end
        return Dict("choices" => choices,
                    "solution" => solution,
                    "candidate" => selected_row,
                    "results" => results,
                    "score" => size(choices)[1])

    end


    """converts a result to a matrix"""
    function result_2_matrix(result, max_tries=10)
        

        max_tries -= 1 #the last one will be popped...

        inputs = copy(result["choices"])
        results = copy(result["results"])
        score = result["score"]

        #delete last entry
        solution = pop!(inputs)
        pop!(results)

        score = result["score"]
        if score > 10
            throw(error("too many tries"))
        end

        res2vec(r::RESULT) = [r.cols, r.pos]

        mem = Any[]
        #the first columns have the score and the solution
        col = zeros(Int8, 1, COLUMNS + 1)
        col[1, 1] = score
        col[1, 2:end] = solution
        push!(mem, col)

        #write one array containing vectors consisting of input and result
        while length(inputs) > 0
            in = pop!(inputs)
            out = pop!(results)
            col = zeros(Int8, 1, COLUMNS + 2)
            col[1, 1:COLUMNS] = in
            col[1, COLUMNS+1:end] = res2vec(out)'

            push!(mem, col)
        end
        vec = collect(Iterators.flatten(mem)) 

        matrix = zeros(Int8, 1, (max_tries)* (COLUMNS + 2) + COLUMNS + 1)
        matrix[1, 1:length(vec)] = vec'

        return matrix
    end

    function result_generator(fname, multiplicity = 1, max_tries = 6)
        """solves all combinations several times (defined by multiplicity) and saves the tries to a file (fname). 
        All Tries with more than max_tries are omitted"""
        
        @time begin
            open(fname, "w") do file

                for sol in CANDIDATES
                    #find a solution for every individual possibility
                    counter = 0
                    while counter < multiplicity
                        res = auto_solver(sol)#solve
                        if res["score"] <= max_tries
                            flat_res = result_2_matrix(res, max_tries)
                            write(file, string(flat_res))
                            write(file, "\n")
                            counter += 1
                        end #try again otherwise
                    end
                end

            end
        end
    end

    """reads a result file an returns it as a matrix.
        One row is one approach to solve it.
        First column is the score (i.e. number of tries)
        then comes the solution.
        All invidiual tries are first listing the input and then the grading.
        The final input (i.e. the solution) is omitted"""
    function read_results(fname)
        

        mem = Any[]
        for line in eachline(fname)
            line = line[5:end-1] #cut  Int8[ and ]
            line = filter.(isdigit, collect.(line)) #remove everything that is not a number
            line = [parse(Int8, c) for c in line ]
            push!(mem, copy(line'))
        end

        return mem
    end

    """play against the machine"""
    function play()
        mem = Any[]
    
        println("##############################################################")
        println("########starting Mastermind. Please find out the code.########")
        println("##############################################################")
        function print_all()
            for i = 1:30
                println("")
            end
            for line in mem
                show(line)
            end
        end
        function read_input()
            while true
                in_list = 0 #just to have it in the toplevel namespace
                try
                    in_str = readline()
                    in_str = filter.(isdigit, collect.(in_str)) #remove everything that is not a number
                    in_list = [parse(Int8, c) for c in in_str ]
                catch
                    println("Please only use integers")
                else
                    if maximum(in_list) == minimum(in_list) == 0
                        return 0
                    elseif (maximum(in_list) > COLORS) | (minimum(in_list) < 1)
                        println("please enter numbers between 1 and "*string(COLORS))
                    elseif size(in_list)[1] == COLUMNS
                        input = RAW_LINE(vec(in_list))
                        res = grade_result(input, solution)
                        line = LINE(input, res)
                        return line
                    else
                        println("please enter "*string(COLUMNS)*" digits")
                    end
                end
            end
        end
    
        solution = random_result()
        print(solution)
    
        result = false
        while result == false
            code_line = read_input() #includes graded result
            #code_line = LINE(new_code)
            try
                #code_line.result = grade_result(code_line.code, solution)
                result = code_line.result
                push!(mem, code_line)
                print_all()
            catch error
                println("exit")
                throw(error)
                return 0
            end
        end
    
        println("CONTRATULATIONS, you solved the code :-)")
    
    end

end
