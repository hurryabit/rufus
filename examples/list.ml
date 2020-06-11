let nil = { isEmpty = true } in
let cons = fun hd tl -> { isEmpty = false; head = hd; tail = tl } in
let rec foldr = fun f z xs ->
  if xs.isEmpty then
    z
  else
    f xs.head (foldr f z xs.tail)
in
let sum = foldr (fun x s -> x+s) 0 in
let map = fun f -> foldr (fun x ys -> cons (f x) ys) nil in
let rec upto_aux = fun acc n -> if n < 0 then acc else upto_aux (cons n acc) (n-1) in
let upto = upto_aux nil in
sum (map (fun x -> x*x) (upto 5))
