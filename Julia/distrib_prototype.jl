using ParallelUtilities
using SharedArrays
using Distributed
using ParallelUtilities
using BenchmarkTools


sleeptime=0.001



function loop(n=10)
    a = zeros(n)
    for i = 1:n
        sleep(sleeptime)
        a[i] = i
    end
    #println("done")
    return a
end
"parallel loop: 1"
function ploop1(n = 10)
    pids = ParallelUtilities.workers_myhost()
    #pids = [1; 2; 3 ; 4]
    a = SharedArray{Int}((n,), pids = pids)

    @sync @distributed for i = 1:n
        sleep(sleeptime)
        a[i] = i
    end
    #println("done")
end
"parallel loop: 2"
function ploop2(n = 10)
    pids = ParallelUtilities.workers_myhost()
    a = SharedArray{Int}((n,), pids = pids)

    for i = 1:n
        sleep(sleeptime)
        a[i] = i
    end
    println("done")
end
@benchmark loop()

@benchmark ploop1()

addprocs(10)

@everywhere using ParallelUtilities
@everywhere using SharedArrays
@everywhere using Distributed

@benchmark ploop1()