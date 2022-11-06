import Base.== #for overloading the operator
using Random
using CSV

############################################################################
#####################    GLOBAL VARIABLES   ################################
############################################################################

COLORS  = 6
COLUMNS = 4


############################################################################
###############     CODE           #########################################
############################################################################


random_result() = rand(1 : COLORS, (1,COLUMNS))

mutable struct LINE
    line :: Matrix{Int8}
end

mutable struct RESULT
    pos  :: Int
    cols :: Int
end

# overload the == operator
Base.:(==)(c::RESULT, d::RESULT) = ((c.pos == d.pos) & (c.cols == d.cols)) 
Base.:(!=)(c::RESULT, d::RESULT) = (!(c == d)) 
Base.:(==)(c::RESULT, d::Bool)   = (d ? c.pos == COLUMNS : c.pos < COLUMNS) #the correct result is true
Base.:(!=)(c::RESULT, d::Bool)   = (!(c == d)) #the correct result is true


function grade_result(input, solution)

    in = copy(input)
    sol = copy(solution)

    correct_positions = 0
    correct_colors = 0


    for i = 1: COLUMNS
        if in[i] == sol[i]
            correct_positions += 1
            in[i] = 0
            sol[i] = 0
        end
    end

    if correct_positions == 4
        return RESULT(4, 0)
    end

    for i = 1: COLUMNS, j = 1: COLUMNS
        if in[i] == 0 || sol[j] == 0
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


function array_list()
    liste = full_list()
    rows, cols = size(liste)
    output = Any[]

    for i = 1:rows
        push!(output, liste[i, :])
    end

    return output
end

CANDIDATES = array_list()

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

    #flattens an array of vectors to an arry
    #vec = collect(Iterators.flatten(inputs)) 

    res2vec(r::RESULT) = [r.cols, r.pos]
    #res = map(res2vec, results) #list comprehension is an alternative
    #res = collect(Iterators.flatten(res))

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

    #cols = (COLUMNS + 2) * max_tries + index

    #the other columns have all the tries
    #matrix = zeros(Int8, 1, cols)
    #matrix[1, 1:size(vec)[1]] = vec'
    #matrix[1, (max_tries * COLUMNS + 1):end ] = res'

    return matrix
end

function result_generator(fname, multiplicity = 1, max_tries = 7)
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