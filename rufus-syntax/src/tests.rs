use super::*;

use insta::assert_snapshot;

fn parse(input: &str) -> ParseResult {
    let parser = Parser::new(input);
    return parser.parse(rules::root);
}

fn dump_errors(errors: &Vec<ParseError>) -> String {
    let mut buffer = String::new();
    for error in errors {
        buffer.push_str(&format!("{:?}\n", error));
    }
    buffer
}

#[test]
fn empty() {
    let result = parse("");
    assert_snapshot!(dump_syntax(result.syntax, false), @r#"
    ROOT@0..0
      ERROR@0..0
    "#);
    assert_snapshot!(dump_errors(&result.errors), @r#"
    ParseError { span: 0..0, found: EOF, expected: {FUN, LET, IF, TRUE, FALSE, LPAREN, ID_LOWER, NAT_LIT}, rule: "EXPR" }
    "#);
}

#[test]
fn fun_simple() {
    let result = parse("fun x -> (x + 1)");
    assert_snapshot!(dump_syntax(result.syntax, false), @r##"
  ROOT@0..16
    FUN_EXPR@0..16
      #FUN@0..3 "fun"
      PARAM_LIST@3..6
        PARAM@4..5
          #ID_LOWER@4..5 "x"
      #ARROW@6..8 "->"
      PAREN_EXPR@9..16
        #LPAREN@9..10 "("
        BINOP_EXPR@10..15
          VAR_EXPR@10..11
            #ID_LOWER@10..11 "x"
          BINOP@12..13
            #PLUS@12..13 "+"
          LIT_EXPR@14..15
            #NAT_LIT@14..15 "1"
        #RPAREN@15..16 ")"
  "##);
    assert_snapshot!(dump_errors(&result.errors), @r"");
}

#[test]
fn fun_no_params() {
    let result = parse("fun   -> (x + 1)");
    assert_snapshot!(dump_syntax(result.syntax, false), @r##"
  ROOT@0..16
    FUN_EXPR@0..16
      #FUN@0..3 "fun"
      PARAM_LIST@3..6
      #ARROW@6..8 "->"
      PAREN_EXPR@9..16
        #LPAREN@9..10 "("
        BINOP_EXPR@10..15
          VAR_EXPR@10..11
            #ID_LOWER@10..11 "x"
          BINOP@12..13
            #PLUS@12..13 "+"
          LIT_EXPR@14..15
            #NAT_LIT@14..15 "1"
        #RPAREN@15..16 ")"
  "##);
    assert_snapshot!(dump_errors(&result.errors), @r#"
  "#);
}

#[test]
fn fun_no_arrow() {
    let result = parse("fun x    (x + 1)");
    assert_snapshot!(dump_syntax(result.syntax, false), @r##"
  ROOT@0..16
    FUN_EXPR@0..16
      #FUN@0..3 "fun"
      PARAM_LIST@3..9
        PARAM@4..5
          #ID_LOWER@4..5 "x"
      ERROR@9..9
      PAREN_EXPR@9..16
        #LPAREN@9..10 "("
        BINOP_EXPR@10..15
          VAR_EXPR@10..11
            #ID_LOWER@10..11 "x"
          BINOP@12..13
            #PLUS@12..13 "+"
          LIT_EXPR@14..15
            #NAT_LIT@14..15 "1"
        #RPAREN@15..16 ")"
  "##);
    assert_snapshot!(dump_errors(&result.errors), @r#"
    ParseError { span: 9..10, found: LPAREN, expected: {ARROW}, rule: "FUN_EXPR" }
    "#);
}

#[test]
fn fun_bad_body() {
    let result = parse("fun x -> (x +  )");
    assert_snapshot!(dump_syntax(result.syntax, false), @r##"
    ROOT@0..16
      FUN_EXPR@0..16
        #FUN@0..3 "fun"
        PARAM_LIST@3..6
          PARAM@4..5
            #ID_LOWER@4..5 "x"
        #ARROW@6..8 "->"
        PAREN_EXPR@9..16
          #LPAREN@9..10 "("
          BINOP_EXPR@10..15
            VAR_EXPR@10..11
              #ID_LOWER@10..11 "x"
            BINOP@12..13
              #PLUS@12..13 "+"
            ERROR@15..15
          #RPAREN@15..16 ")"
    "##);
    assert_snapshot!(dump_errors(&result.errors), @r#"
    ParseError { span: 15..16, found: RPAREN, expected: {TRUE, FALSE, LPAREN, ID_LOWER, NAT_LIT}, rule: "ATOM_EXPR" }
    "#);
}

#[test]
fn fun_no_body() {
    let result = parse("fun x ->");
    assert_snapshot!(dump_syntax(result.syntax, false), @r##"
    ROOT@0..8
      FUN_EXPR@0..8
        #FUN@0..3 "fun"
        PARAM_LIST@3..6
          PARAM@4..5
            #ID_LOWER@4..5 "x"
        #ARROW@6..8 "->"
        ERROR@8..8
    "##);
    assert_snapshot!(dump_errors(&result.errors), @r#"
    ParseError { span: 8..8, found: EOF, expected: {FUN, LET, IF, TRUE, FALSE, LPAREN, ID_LOWER, NAT_LIT}, rule: "EXPR" }
    "#);
}

#[test]
fn let_simple() {
    let result = parse("let     x = 1 + 2 in x + 1");
    assert_snapshot!(dump_syntax(result.syntax, false), @r##"
    ROOT@0..26
      LET_EXPR@0..26
        #LET@0..3 "let"
        LET_MOD@3..8
        LET_VAR@8..9
          #ID_LOWER@8..9 "x"
        #ASSIGN@10..11 "="
        BINOP_EXPR@12..18
          LIT_EXPR@12..13
            #NAT_LIT@12..13 "1"
          BINOP@14..15
            #PLUS@14..15 "+"
          LIT_EXPR@16..17
            #NAT_LIT@16..17 "2"
        #IN@18..20 "in"
        BINOP_EXPR@21..26
          VAR_EXPR@21..22
            #ID_LOWER@21..22 "x"
          BINOP@23..24
            #PLUS@23..24 "+"
          LIT_EXPR@25..26
            #NAT_LIT@25..26 "1"
    "##);
    assert_snapshot!(dump_errors(&result.errors), @"");
}

#[test]
fn let_rec() {
    let result = parse("let rec x = 1 + 2 in x + 1");
    assert_snapshot!(dump_syntax(result.syntax, false), @r##"
    ROOT@0..26
      LET_EXPR@0..26
        #LET@0..3 "let"
        LET_MOD@3..7
          #REC@4..7 "rec"
        LET_VAR@7..9
          #ID_LOWER@8..9 "x"
        #ASSIGN@10..11 "="
        BINOP_EXPR@12..18
          LIT_EXPR@12..13
            #NAT_LIT@12..13 "1"
          BINOP@14..15
            #PLUS@14..15 "+"
          LIT_EXPR@16..17
            #NAT_LIT@16..17 "2"
        #IN@18..20 "in"
        BINOP_EXPR@21..26
          VAR_EXPR@21..22
            #ID_LOWER@21..22 "x"
          BINOP@23..24
            #PLUS@23..24 "+"
          LIT_EXPR@25..26
            #NAT_LIT@25..26 "1"
    "##);
    assert_snapshot!(dump_errors(&result.errors), @"");
}

#[test]
fn let_no_var() {
    let result = parse("let       = 1 + 2 in x + 1");
    assert_snapshot!(dump_syntax(result.syntax, false), @r##"
    ROOT@0..26
      LET_EXPR@0..26
        #LET@0..3 "let"
        LET_MOD@3..10
        LET_VAR@10..10
        #ASSIGN@10..11 "="
        BINOP_EXPR@12..18
          LIT_EXPR@12..13
            #NAT_LIT@12..13 "1"
          BINOP@14..15
            #PLUS@14..15 "+"
          LIT_EXPR@16..17
            #NAT_LIT@16..17 "2"
        #IN@18..20 "in"
        BINOP_EXPR@21..26
          VAR_EXPR@21..22
            #ID_LOWER@21..22 "x"
          BINOP@23..24
            #PLUS@23..24 "+"
          LIT_EXPR@25..26
            #NAT_LIT@25..26 "1"
    "##);
    assert_snapshot!(dump_errors(&result.errors), @r#"
    ParseError { span: 10..11, found: ASSIGN, expected: {ID_LOWER}, rule: "LET_VAR" }
    "#);
}

#[test]
fn let_no_assign() {
    let result = parse("let     x   1 + 2 in x + 1");
    assert_snapshot!(dump_syntax(result.syntax, false), @r##"
    ROOT@0..26
      LET_EXPR@0..26
        #LET@0..3 "let"
        LET_MOD@3..8
        LET_VAR@8..9
          #ID_LOWER@8..9 "x"
        ERROR@12..12
        BINOP_EXPR@12..18
          LIT_EXPR@12..13
            #NAT_LIT@12..13 "1"
          BINOP@14..15
            #PLUS@14..15 "+"
          LIT_EXPR@16..17
            #NAT_LIT@16..17 "2"
        #IN@18..20 "in"
        BINOP_EXPR@21..26
          VAR_EXPR@21..22
            #ID_LOWER@21..22 "x"
          BINOP@23..24
            #PLUS@23..24 "+"
          LIT_EXPR@25..26
            #NAT_LIT@25..26 "1"
    "##);
    assert_snapshot!(dump_errors(&result.errors), @r#"
    ParseError { span: 12..13, found: NAT_LIT, expected: {ASSIGN}, rule: "LET_EXPR" }
    "#);
}

#[test]
fn let_no_bindee() {
    let result = parse("let     x =       in x + 1");
    assert_snapshot!(dump_syntax(result.syntax, false), @r##"
    ROOT@0..26
      LET_EXPR@0..26
        #LET@0..3 "let"
        LET_MOD@3..8
        LET_VAR@8..9
          #ID_LOWER@8..9 "x"
        #ASSIGN@10..11 "="
        ERROR@18..18
        #IN@18..20 "in"
        BINOP_EXPR@21..26
          VAR_EXPR@21..22
            #ID_LOWER@21..22 "x"
          BINOP@23..24
            #PLUS@23..24 "+"
          LIT_EXPR@25..26
            #NAT_LIT@25..26 "1"
    "##);
    assert_snapshot!(dump_errors(&result.errors), @r#"
    ParseError { span: 18..20, found: IN, expected: {FUN, LET, IF, TRUE, FALSE, LPAREN, ID_LOWER, NAT_LIT}, rule: "EXPR" }
    "#);
}

#[test]
fn let_bad_bindee1() {
    let result = parse("let     x =   + 2 in x + 1");
    assert_snapshot!(dump_syntax(result.syntax, false), @r##"
    ROOT@0..26
      LET_EXPR@0..26
        #LET@0..3 "let"
        LET_MOD@3..8
        LET_VAR@8..9
          #ID_LOWER@8..9 "x"
        #ASSIGN@10..11 "="
        ERROR@14..16
          #PLUS@14..15 "+"
        LIT_EXPR@16..17
          #NAT_LIT@16..17 "2"
        #IN@18..20 "in"
        BINOP_EXPR@21..26
          VAR_EXPR@21..22
            #ID_LOWER@21..22 "x"
          BINOP@23..24
            #PLUS@23..24 "+"
          LIT_EXPR@25..26
            #NAT_LIT@25..26 "1"
    "##);
    assert_snapshot!(dump_errors(&result.errors), @r#"
    ParseError { span: 14..15, found: PLUS, expected: {FUN, LET, IF, TRUE, FALSE, LPAREN, ID_LOWER, NAT_LIT}, rule: "EXPR" }
    "#);
}

#[test]
fn let_bad_bindee2() {
    let result = parse("let     x = 1 +   in x + 1");
    assert_snapshot!(dump_syntax(result.syntax, false), @r##"
    ROOT@0..26
      LET_EXPR@0..26
        #LET@0..3 "let"
        LET_MOD@3..8
        LET_VAR@8..9
          #ID_LOWER@8..9 "x"
        #ASSIGN@10..11 "="
        BINOP_EXPR@12..18
          LIT_EXPR@12..13
            #NAT_LIT@12..13 "1"
          BINOP@14..15
            #PLUS@14..15 "+"
          ERROR@18..18
        #IN@18..20 "in"
        BINOP_EXPR@21..26
          VAR_EXPR@21..22
            #ID_LOWER@21..22 "x"
          BINOP@23..24
            #PLUS@23..24 "+"
          LIT_EXPR@25..26
            #NAT_LIT@25..26 "1"
    "##);
    assert_snapshot!(dump_errors(&result.errors), @r#"
    ParseError { span: 18..20, found: IN, expected: {TRUE, FALSE, LPAREN, ID_LOWER, NAT_LIT}, rule: "ATOM_EXPR" }
    "#);
}

#[test]
fn let_no_in() {
    let result = parse("let     x = 1 + 2    x + 1");
    assert_snapshot!(dump_syntax(result.syntax, false), @r##"
    ROOT@0..26
      LET_EXPR@0..26
        #LET@0..3 "let"
        LET_MOD@3..8
        LET_VAR@8..9
          #ID_LOWER@8..9 "x"
        #ASSIGN@10..11 "="
        BINOP_EXPR@12..21
          LIT_EXPR@12..13
            #NAT_LIT@12..13 "1"
          BINOP@14..15
            #PLUS@14..15 "+"
          LIT_EXPR@16..17
            #NAT_LIT@16..17 "2"
        ERROR@21..21
        BINOP_EXPR@21..26
          VAR_EXPR@21..22
            #ID_LOWER@21..22 "x"
          BINOP@23..24
            #PLUS@23..24 "+"
          LIT_EXPR@25..26
            #NAT_LIT@25..26 "1"
    "##);
    assert_snapshot!(dump_errors(&result.errors), @r#"
    ParseError { span: 21..22, found: ID_LOWER, expected: {IN}, rule: "LET_EXPR" }
    "#);
}

#[test]
fn let_no_body() {
    let result = parse("let     x = 1 + 2 in");
    assert_snapshot!(dump_syntax(result.syntax, false), @r##"
    ROOT@0..20
      LET_EXPR@0..20
        #LET@0..3 "let"
        LET_MOD@3..8
        LET_VAR@8..9
          #ID_LOWER@8..9 "x"
        #ASSIGN@10..11 "="
        BINOP_EXPR@12..18
          LIT_EXPR@12..13
            #NAT_LIT@12..13 "1"
          BINOP@14..15
            #PLUS@14..15 "+"
          LIT_EXPR@16..17
            #NAT_LIT@16..17 "2"
        #IN@18..20 "in"
        ERROR@20..20
    "##);
    assert_snapshot!(dump_errors(&result.errors), @r#"
    ParseError { span: 20..20, found: EOF, expected: {FUN, LET, IF, TRUE, FALSE, LPAREN, ID_LOWER, NAT_LIT}, rule: "EXPR" }
    "#);
}

#[test]
fn let_bad_body1() {
    let result = parse("let     x = 1 + 2 in   + 1");
    assert_snapshot!(dump_syntax(result.syntax, false), @r##"
    ROOT@0..26
      LET_EXPR@0..26
        #LET@0..3 "let"
        LET_MOD@3..8
        LET_VAR@8..9
          #ID_LOWER@8..9 "x"
        #ASSIGN@10..11 "="
        BINOP_EXPR@12..18
          LIT_EXPR@12..13
            #NAT_LIT@12..13 "1"
          BINOP@14..15
            #PLUS@14..15 "+"
          LIT_EXPR@16..17
            #NAT_LIT@16..17 "2"
        #IN@18..20 "in"
        ERROR@23..25
          #PLUS@23..24 "+"
        LIT_EXPR@25..26
          #NAT_LIT@25..26 "1"
    "##);
    assert_snapshot!(dump_errors(&result.errors), @r#"
    ParseError { span: 23..24, found: PLUS, expected: {FUN, LET, IF, TRUE, FALSE, LPAREN, ID_LOWER, NAT_LIT}, rule: "EXPR" }
    "#);
}

#[test]
fn let_bad_body2() {
    let result = parse("let     x = 1 + 2 in x +");
    assert_snapshot!(dump_syntax(result.syntax, false), @r##"
    ROOT@0..24
      LET_EXPR@0..24
        #LET@0..3 "let"
        LET_MOD@3..8
        LET_VAR@8..9
          #ID_LOWER@8..9 "x"
        #ASSIGN@10..11 "="
        BINOP_EXPR@12..18
          LIT_EXPR@12..13
            #NAT_LIT@12..13 "1"
          BINOP@14..15
            #PLUS@14..15 "+"
          LIT_EXPR@16..17
            #NAT_LIT@16..17 "2"
        #IN@18..20 "in"
        BINOP_EXPR@21..24
          VAR_EXPR@21..22
            #ID_LOWER@21..22 "x"
          BINOP@23..24
            #PLUS@23..24 "+"
          ERROR@24..24
    "##);
    assert_snapshot!(dump_errors(&result.errors), @r#"
    ParseError { span: 24..24, found: EOF, expected: {TRUE, FALSE, LPAREN, ID_LOWER, NAT_LIT}, rule: "ATOM_EXPR" }
    "#);
}

#[test]
fn arith_simple() {
    let result = parse("1 + 2 * 3 + 4");
    assert_snapshot!(dump_syntax(result.syntax, false), @r##"
    ROOT@0..13
      BINOP_EXPR@0..13
        BINOP_EXPR@0..10
          LIT_EXPR@0..1
            #NAT_LIT@0..1 "1"
          BINOP@2..3
            #PLUS@2..3 "+"
          BINOP_EXPR@4..9
            LIT_EXPR@4..5
              #NAT_LIT@4..5 "2"
            BINOP@6..7
              #STAR@6..7 "*"
            LIT_EXPR@8..9
              #NAT_LIT@8..9 "3"
        BINOP@10..11
          #PLUS@10..11 "+"
        LIT_EXPR@12..13
          #NAT_LIT@12..13 "4"
    "##);
    assert_snapshot!(dump_errors(&result.errors), @r"");
}
