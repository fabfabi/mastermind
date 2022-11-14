include("mastermind.jl")
using .MASTERMIND
using DataFrames
using Flux
using Flux: train!, onecold, mean
using BSON: @save

THRESHOLD = 0.8 # --> x % Training data
multiplicity = 6



decisions = DataFrame(code = CANDIDATES)


"""
write_purpose(num :: Float64) = num > THRESHOLD ? "test" : "train"
decide_purpose(in) = write_purpose.(rand(size(in)...))
df.purpose = decide_purpose(df.code)
"""

decision(in)=  rand() > THRESHOLD ? "test" : "train"
decisions.purpose = broadcast(decision, decisions.code)
function read_data(fname)
    data_raw = read_results(fname)

    full_data = data_raw[1]

    for i = 2 : size(data_raw)[1]
        full_data = vcat(full_data, data_raw[i])
    end
    scores = full_data[:, 1]
    solutions = full_data[:, 2:COLUMNS + 1]
    inputs = full_data[:, COLUMNS+2:end]



    ohb = Flux.onehotbatch(inputs', 0:COLORS)
    x_data = Flux.flatten(ohb)

    sohb = Flux.onehotbatch(solutions', 1:COLORS)
    y_data = Flux.flatten(sohb)
    return x_data, y_data
end

x_train, y_train = read_data("training_results_10.txt")

x_test, y_test_raw = read_data("testing_results_10.txt")

model = Chain(
    Dense(210, 64, relu),
    Dense(64, COLUMNS * COLORS),
    softmax
)

loss(x,y) = Flux.crossentropy(model(x), y)
ps = Flux.params(model)

learning_rate = 0.01

opt = Flux.ADAM(learning_rate)
loss_history = []
epochs = 100

for epoch in 1:epochs
    Flux.train!(loss, ps, [(x_train, y_train)], opt)
    train_loss = loss(x_train, y_train)
    push!(loss_history, train_loss)
    println("Epoch = $epoch : Train Loss = $train_loss")
end

y_hat_raw = model(x_test)

"""one cold just returns the index with the highest probability. We need to split the data into 
4 columns and re-assemble the matrix"""
function multi_onecold(data)
    return_data = onecold(data[1:COLORS, :])'
    for i = 1 : COLUMNS - 1
        return_data = vcat(return_data, onecold(data[i*COLORS + 1:(i+1) * COLORS, :])')
    end
    return return_data
end

y_hat = multi_onecold(y_hat_raw)

y_test = multi_onecold(y_test_raw)

mean(y_hat .== y_test)

check = [y_hat[i] == y_test for i in 1:size(y_hat)[2]]

index = collect(1:size(y_hat)[2])

check_display = [index y_hat' y_test' check]

vscodedisplay(check_display)

@save "20221114_first_model.bson" model