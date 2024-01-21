using Distributed
addprocs(10)


@everywhere include("mastermind.jl")
@everywhere using .MASTERMIND
using Test
using Logging
using DataStructures
using Distributed
using ParallelUtilities
using SharedArrays
using BenchmarkTools


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
function minmax_candidate(candidates :: Array{RAW_LINE}) :: RAW_LINE
    best_val = 1000000
    best_candidate = "None"
    pids = ParallelUtilities.workers_myhost()
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
end

@test minmax_candidate(raw_line_list()) == RAW_LINE(vec([1 1 2 2]))

@benchmark minmax_candidate(raw_line_list())
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