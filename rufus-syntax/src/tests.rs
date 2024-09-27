use super::*;

use insta::assert_snapshot;

fn parse(input: &str) -> ParseResult {
    let parser = Parser::new(input);
    return parser.parse();
}

fn dump_errors(errors: &Vec<ParseError>) -> String {
    let mut buffer = String::new();
    for error in errors {
        buffer.push_str(&format!("{:?}", error));
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
    ParseError { span: 0..0, found: EOF, expected: {FUN, LET, IF, TRUE, FALSE, LPAREN, ID_LOWER, NAT_LIT}, rule: "ROOT" }
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
  assert_snapshot!(dump_errors(&result.errors), @r#"
  "#);
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
  assert_snapshot!(dump_errors(&result.errors), @r#"ParseError { span: 9..10, found: LPAREN, expected: {ARROW}, rule: "FUN_EXPR" }"#);
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
  assert_snapshot!(dump_errors(&result.errors), @r#"ParseError { span: 15..16, found: RPAREN, expected: {TRUE, FALSE, LPAREN, ID_LOWER, NAT_LIT}, rule: "atom_expr" }"#);
}

#[ignore]
#[test]
fn fun_no_body() {
  let result = parse("fun x ->");
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
  assert_snapshot!(dump_errors(&result.errors), @r#"
  "#);
}
