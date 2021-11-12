(* This is a simple example to
   demonstrate the syntax of rufus.
*)
let twice = fun f x -> f (f x) in
let inc = fun x -> x + 1 in
twice inc 0
