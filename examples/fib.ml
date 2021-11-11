let rec fib = fun n ->
    if n <= 1 then
        1
    else
        fib (n-2) + fib (n-1)
in
fib 9
