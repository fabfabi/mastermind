using Distributed
addprocs(15)

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



global_logger(Logging.SimpleLogger(Debug))
global_logger(Logging.SimpleLogger(Info))
"""
find the best candidate that creates the minimum number of future candidates per result
"""
function minmax_candidate(candidates :: Array{RAW_LINE}) :: RAW_LINE

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

    L = length(candidates)
    if L <= 15
        return minmax_candidate_ref(candidates)
    else
        return minmax_candidate_pmap(candidates)
    end
end

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

"""run all possible combinations through a solver and return the results"""
function check_solver(solver :: Base.Callable)
    candidates = raw_line_list()
    data = zeros(length(candidates), 1)
    
    @time for (index, candidate) in enumerate(candidates)
        
        data[index, 1] = solver(candidate)
        println(string(index)*" "*string(data[index, 1]))
    end

    return data
end