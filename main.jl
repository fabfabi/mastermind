import Base.== #for overloading the operator

############################################################################
#####################    GLOBAL VARIABLES   ################################
############################################################################

COLORS  = 6
COLUMNS = 4


############################################################################
###############     CODE           #########################################
############################################################################


random_result() = rand(1 : COLORS, (1,COLUMNS))

mutable struct result
    correct_pos  :: Int
    correct_cols :: Int
end
# overload the == operator
Base.:(==)(c::result, d::result) = ((c.correct_pos == d.correct_pos) & (c.correct_cols == d.correct_cols)) 

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
    return result(correct_positions, correct_colors)
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


mutable struct asdf
    a::String
    b::Int
end

b = asdf("text", 5)