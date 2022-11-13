include("mastermind.jl")
using .MASTERMIND
using DataFrames
using Flux

THRESHOLD = 0.8 # --> x % Training data
multiplicity = 6

data_raw = read_results("training_results_10.txt")

decisions = DataFrame(code = CANDIDATES)


"""
write_purpose(num :: Float64) = num > THRESHOLD ? "test" : "train"
decide_purpose(in) = write_purpose.(rand(size(in)...))
df.purpose = decide_purpose(df.code)
"""

decision(in)=  rand() > THRESHOLD ? "test" : "train"

decisions.purpose = broadcast(decision, decisions.code)


full_data = data_raw[1]

for i = 2 : size(data_raw)[1]
    full_data = vcat(full_data, data_raw[i])
end
scores = full_data[:, 1]
solutions = full_data[:, 2:COLUMNS + 1]
inputs = full_data[:, COLUMNS+2:end]



ohb = Flux.onehotbatch(inputs', 0:9)
ohbf = Flux.flatten(ohb)

sohb = Flux.onehotbatch(solutions', 0:9)
sobf = Flux.flatten(sohb)