mutable struct Foo
    #bei diesem Struct kann man die funktionen set_val und get_val via f.set_val aurfurenf
    value :: Int64
    set_val
    get_val
    function Foo(x)
        f = new(x)
        f.set_val = (y) -> (f.value = y)
        f.get_val = () -> f.value
        return f
    end
    #dies ist von der Performanz langsamer, als 
end


function testfun(x)
    2 * x # funktioniert genauso wie return 2 * x
end

mutable struct fastFoo #kommentar
    #das gleiche wie oben in schneller
    value :: Int64
end

function set_val!(f::fastFoo, s::Symbol, v)
    if s == :value
        f.value = v
    else
        error("unknown property $s")
    end
end

function get_val!(f::fastFoo, s::Symbol)
    if s == :value
        return f.value
    else
        error("unknown property $s")
    end

end

@time begin
    tester = Foo(0)
    ref = Foo(-1)
    for i = 1:10000
        tester.set_val(i)
        ref = tester.get_val()
    end
end

@time begin
    tester = fastFoo(0)
    ref = fastFoo(-1)
    for i = 1:10000
        set_val!(tester, :value, i)
        ref = get_val!(tester, :value)
    end
    #this is 10 times as fast as the above one
end

tester = Foo(0)
using BenchmarkTools

@benchmark begin
    tester.set_val(1)
    tester.get_val()
end

tester2 = fastFoo(0)

@benchmark begin
    set_val!(tester2, :value, 1)
    get_val!(tester2, :value)
end