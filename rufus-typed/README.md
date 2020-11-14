# rufus-typed

This is the core library for a typed version of the rufus language, containing the parser and the interpreter.


## Typing rules

Type rules for expressions:
```
  E |- e => t
-------------------------------------------------- CheckInfer
  E |- e <= t


  type T<A1, ..., An> = t
  E |- e => T[s1, ..., sn]
-------------------------------------------------- SynInfer
  E |- e => t[s1/A1, ..., sn/An]


  type T<A1, ..., An> = t
  E |- e <= t[s1/A1, ..., sn/An]
-------------------------------------------------- SynCheck
  E |- e <= T[s1, ..., sn]


  (x: t) in E
-------------------------------------------------- Var
  E |- x => t


  n in {0, 1, -1, 2, -2, ...}
-------------------------------------------------- LitInt
  E |- n => Int


  b in {true, false}
-------------------------------------------------- LitBool
  E |- b => Bool


  x1, ..., xn mutually distinct
  E, x1: s1, ..., xn: sn |- e => t
-------------------------------------------------- LamInfer
  E |- fn (x1: s1, ..., xn: sn) { e }
    => (s1, ..., sn) -> t


  x1, ..., xn mutually distinct
  si ~ si'
  E, x1: s1, ..., sn: tn |- e <= t
-------------------------------------------------- LamCheck
  E |- fn (x1, ..., xi: si', ..., xn) { e }
    <= (s1, ..., sn) -> t


  fn f<A1, ..., Am>(x1: s1, ..., xn: sn) -> t
-------------------------------------------------- FuncInst
  E |- f@<u1, ..., um>
    => ((s1, ..., sn) -> t)[u1/A1, ..., um/Am]


  E |- f => (s1, ..., sn) -> t
  E |- e1 <= s1
  ...
  E |- en <= sn
-------------------------------------------------- App
  E |- f(e1, ..., en) => t


  § in {+, -, *, /}
  E |- e1 <= Int
  E |- e2 <= Int
-------------------------------------------------- BinOpArith
  E |- e1 § e2 => Int


  § in {==, !=, <, <=, =>, >}
  E |- e1 => t
  E |- e2 <= t
-------------------------------------------------- BinOpCmp
  E |- e1 § e2 => Bool


  E |- e1 => s
  E, x: s |- e2 => t
-------------------------------------------------- LetInferInfer
  E |- let x = e1 in e2 => t


  E |- e1 <= s
  E, x: s |- e2 => t
-------------------------------------------------- LetCheckInfer
  E |- let x: s = e1 in e2 => t


  E |- e1 => s
  E, x: s |- e2 <= t
-------------------------------------------------- LetInferCheck
  E |- let x = e1 in e2 <= t


  E |- e1 <= s
  E, x: s |- e2 <= t
-------------------------------------------------- LetCheckCheck
  E |- let x: s = e1 in e2 <= t


  E |- e1 <= Bool
  E |- e2 => t
  E |- e3 <= t
-------------------------------------------------- IfInfer
  E |- if e1 { e2 } else { e3 } => t


  E |- e1 <= Bool
  E |- e2 <= t
  E |- e3 <= t
-------------------------------------------------- IfCheck
  E |- if e1 { e2 } else { e3 } <= t


  E |- e1 => t1
  ...
  E |- en => tn
-------------------------------------------------- RecordInfer
  E |- {x1 = e1, ..., xn = en}
    => {x1: t1, ..., xn: tn}


  E |- e1 <= t1
  ...
  E |- en <= tn
-------------------------------------------------- RecordCheck
  E |- {x1 = e1, ..., xn = en}
    <= {x1: t1, ..., xn: tn}


  E |- e => s
  s = {..., a: t, ...}
-------------------------------------------------- Proj
  E |- e.a => t


  .
-------------------------------------------------- VariantWithoutPayload
  E |- c <= [... | c | ...]


  E |- e <= t
-------------------------------------------------- VariantWithPayload
  E |- c(e) <= [... | c(t) | ...]


  E |- e => s
  s = [...]
  E |- branch s { p1 => e1 } => t
  E |- branch s { p2 => e2 } <= t
  ...
  E |- branch s { pn => en } <= t
-------------------------------------------------- MatchInfer
  E |- match e {p1 => e1, ..., pn -> en} => t


  E |- e => s
  s = [...]
  E |- branch s { p1 => e1 } <= t
  ...
  E |- branch s { pn => en } <= t
-------------------------------------------------- MatchCheck
  E |- match e {p1 -> e1, ..., pn -> en} <= t
```

Typing rules for `match` branches `branch s {p => e}`:
```
  E |- e => t
-------------------------------------------------- BranchWithoutPayloadInfer
  E |- branch [... | c | ...] { c => e }
    => t


  E |- e <= t
-------------------------------------------------- BranchWithoutPayloadCheck
  E |- branch [... | c | ...] { c => e }
    <= t


  E, x: s |- e => t
-------------------------------------------------- BranchWithPayloadInfer
  E |- branch [... | c(s) | ...] { c(x) => e }
    => t


  E, x: s |- e <= t
-------------------------------------------------- BranchWithPayloadCheck
  E |- branch [... | c(s) | ...] { c(x) => e }
    <= t
```

http://davidchristiansen.dk/tutorials/bidirectional.pdf
