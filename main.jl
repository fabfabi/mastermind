using Distributed
addprocs(10)

include("mastermind.jl")
@everywhere include("mastermind.jl")
@everywhere using .MASTERMIND
using Test
using Logging
using DataStructures
using Distributed
using ParallelUtilities
using SharedArrays
using BenchmarkTools
using .MASTERMIND


#=
@everywhere using ParallelUtilities
@everywhere using SharedArrays
@everywhere using Distributed
@everywhere using DataStructures
=#
#@everywhere using Pkg
#@everywhere Pkg.activate("~")
#@everywhere using .MASTERMIND
#@everywhere RAW_LINE = MASTERMIND.RAW_LINE
#@everywhere grade_result = MASTERMIND.grade_result



global_logger(Logging.SimpleLogger(Debug))


""" find the candidate that minimizes the maximum group size per individual result"""
function minmax_candidate_p1(candidates :: Array{RAW_LINE}) :: RAW_LINE
    best_val = 1000000
    best_candidate = "None"
    L = length(candidates)
    if L== 1
        return candidates[1]
    end
    
    for candidate in candidates
        #= 
        mem = SharedArray{RESULT}((L,), pids = pids)

        @sync @distributed for i = 1 : L
            mem[i] = grade_result(candidate, candidates[i])
        end =#

        mem = pmap(x -> grade_result(candidate, x), candidates, distributed=true)

        counter_dict = counter(mem)
        current_val = maximum(values(counter_dict))
        if current_val < best_val
            best_val = current_val
            best_candidate = candidate
        end
    end

    return best_candidate
end # -> 38 ms

function minmax_candidate_p1b(candidates :: Array{RAW_LINE}) :: RAW_LINE
    best_val = 1000000
    best_candidate = "None"
    L = length(candidates)
    if L== 1
        return candidates[1]
    end
    
    for candidate in candidates
        #= 
        mem = SharedArray{RESULT}((L,), pids = pids)

        @sync @distributed for i = 1 : L
            mem[i] = grade_result(candidate, candidates[i])
        end =#

        mem = pmap(x -> grade_result(candidate, x), candidates, distributed=true, batch_size=10)

        counter_dict = counter(mem)
        current_val = maximum(values(counter_dict))
        if current_val < best_val
            best_val = current_val
            best_candidate = candidate
        end
    end

    return best_candidate
end # -> 18 sekunden

function minmax_candidate_pmap(candidates :: Array{RAW_LINE}) :: RAW_LINE

    L = length(candidates)
    if L== 1
        return candidates[1]
    end

    function analyzer(candidate :: RAW_LINE) :: Int

        counter = Dict{RESULT, Int}()

        for c in candidates
            result = grade_result(candidate, c)
            counter[result] = get(counter, result, 0) + 1
        end

        current_val = maximum(values(counter))
        return current_val
    end

    mem = pmap(x -> analyzer(x), candidates, batch_size=36)
    
    best_val = minimum(values(mem))

    for (index, value) in enumerate(mem)
        if value == best_val
            return candidates[index]
        end
    end
end

function minmax_candidate_ref(candidates :: Array{RAW_LINE}) :: RAW_LINE
    best_val = 1000000
    best_candidate = "None"

    if size(candidates)[1] == 1
        return candidates[1]
    end

    #pids = ParallelUtilities.workers_myhost()
    for candidate in candidates
        counter = Dict{RESULT, Int}()

        for c in candidates
            result = grade_result(candidate, c)
            counter[result] = get(counter, result, 0) + 1
        end
        current_val = maximum(values(counter))
        if current_val < best_val
            best_val = current_val
            best_candidate = candidate
        end
    end

    return best_candidate
end

L = length(raw_line_list())

indexes = 1:6:1296

data = zeros(length(indexes), 3)


candidates = raw_line_list()
for (row, i) in enumerate(indexes)
    data[row, 1] = i
    data[row, 2] = @elapsed minmax_candidate_ref(candidates[1:i])
    data[row, 3] = @elapsed minmax_candidate_pmap(candidates[1:i])
    println(string(i)*" "*string(data[row, 2])*" "*string(data[row, 3]) )
end


candidate = candidates[1]
L = length(candidates)
m = Array{RESULT, L}[]
@benchmark begin
    m = pmap(x -> grade_result(candidate, x), candidates)
end

@test minmax_candidate(raw_line_list()) == RAW_LINE(vec([1 1 2 2]))

l = raw_line_list()[1:20]
@benchmark raw_line_list()
@benchmark minmax_candidate_p2(l)
@benchmark minmax_candidate_ref(l)
"""
reduces the list of candidates to only those that create the same result as the input
"""
function reduce_candidates(candidates :: Array{RAW_LINE}, input :: RAW_LINE, result :: RESULT) :: Array{RAW_LINE}
    #rows = size(candidates)[1]
    output = RAW_LINE[]
    for candidate in candidates
        if grade_result(candidate, input) == result
            push!(output, candidate)
        end
    end
    return output
end
@test reduce_candidates(raw_line_list(), RAW_LINE(vec([1, 1, 2, 2])), RESULT(4,0)) == vec([RAW_LINE(vec([1, 1, 2, 2]))])

function minmax_solver(solution :: RAW_LINE) :: Int
    @debug "finding solution for " *string(solution.code)
    counter = 1
    candidates = raw_line_list()
    while length(candidates) > 1
        next_candidate = minmax_candidate(candidates)
        result = grade_result(solution, next_candidate)
        candidates = reduce_candidates(candidates, next_candidate, result)
        if result == true
            @debug string(counter)*" solution found: "*string(next_candidate.code)
            break
        else
            @debug string(counter)*" candidate is "*string(next_candidate.code) * "("*string(length(candidates))*")"
        end
        counter += 1
    end
    

    return counter
end

l = MASTERMIND.raw_line_list()[1:10]