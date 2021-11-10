let twice = fun f x -> f (f x) in
let inc = fun x -> x + 1 in
twice inc 0
