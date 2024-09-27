// This module implements the following context-free grammar with start symbol
// ROOT (rules are indented for the sake of visual grouing only):
//
// ROOT -> EXPR
// EXPR -> FUN_EXPR | LET_EXPR | IF_EXPR | SUM_EXPR
// FUN_EXPR -> "fun" PARAM_LIST "->" EXPR
//   PARAM_LIST -> PARAM*
//   PARAM -> ID_LOWER
// LET_EXPR -> "let" LET_MOD LET_VAR "=" EXPR "in" EXPR
//   LET_MOD -> "rec"?
//   LET_VAR -> ID_LOWER
// IF_EXPR -> "if" EXPR "then" EXPR "else" EXPR
// SUM_EXPR -> PROD_EXPR | SUM_EXPR SUM_OP PROD_EXPR
//   SUM_OP -> "+" | "-"
// PROD_EXPR -> ATOM_EXPR | PROD_EXPR PROD_OP ATOM_EXPR
//   PROD_OP -> "*" | "/"
// ATOM_EXPR -> VAR_EXPR | LIT_EXPR | PAREN_EXPR
// VAR_EXPR -> ID_LOWER
// LIT_EXPR -> NUM_LIT | TRUE | FALSE
// PAREN_EXPR -> "(" EXPR ")"
//
// The resulting CST does not contain nodes for EXPR and ATOM_EXPR but rather
// just their immediate children. SUM_EXPR and PROD_EXPR use a common node
// type BINOP_EXPR. Similarly, SUM_OP and PROD_OP are fused into BINOP.

use crate::kind::{SyntaxKind::*, SyntaxKindSet};
use crate::parser::Parser;

// Token classes.
const ADD_OPS: SyntaxKindSet = SyntaxKindSet::from([PLUS, MINUS]);
const MUL_OPS: SyntaxKindSet = SyntaxKindSet::from([STAR, SLASH]);
const CMP_OPS: SyntaxKindSet = SyntaxKindSet::from([EQ, NE, LT, LE, GT, GE]);
const BIN_OPS: SyntaxKindSet = SyntaxKindSet::union([ADD_OPS, MUL_OPS, CMP_OPS]);
const LITERAL: SyntaxKindSet = SyntaxKindSet::from([NAT_LIT, TRUE, FALSE]);

// First sets.
const FIRST_ATOM_EXPR: SyntaxKindSet =
    SyntaxKindSet::union([SyntaxKindSet::from([ID_LOWER, LPAREN]), LITERAL]);
const FIRST_EXPR: SyntaxKindSet =
    SyntaxKindSet::union([SyntaxKindSet::from([FUN, LET, IF]), FIRST_ATOM_EXPR]);

// Follow sets.
const FOLLOW_EXPR: SyntaxKindSet = SyntaxKindSet::union([
    SyntaxKindSet::from([RPAREN, DOT, IN, THEN, ELSE, EOF]),
    BIN_OPS,
    FIRST_ATOM_EXPR, // Because function application is juxtaposition.
]);
// const FOLLOW_PARAM_LIST: SyntaxKindSet = SyntaxKindSet::from([ARROW]);
// const FOLLOW_PARAM: SyntaxKindSet =
//     SyntaxKindSet::union([FOLLOW_PARAM_LIST, SyntaxKindSet::from([ID_LOWER])]);

pub fn root(parser: &mut Parser) {
    parser.builder.start_node(ROOT.into());
    let mut token = parser.peek();
    if !FIRST_EXPR.contains(token) {
        parser.error(token, FIRST_EXPR, "ROOT");
        parser.builder.start_node(ERROR.into());
        while token != EOF && !FIRST_EXPR.contains(token) {
            parser.consume(token);
            token = parser.peek();
        }
        parser.builder.finish_node();
    }
    if token == EOF {
        parser.builder.finish_node();
        return;
    }
    expr(parser);
    let mut token = parser.peek();
    if token != EOF {
        parser.error(token, EOF.as_set(), "root");
        parser.builder.start_node(ERROR.into());
        while token != EOF {
            parser.consume(token);
            token = parser.peek();
        }
        parser.builder.finish_node();
    }
    parser.builder.finish_node();
}

// EXPR -> FUN_EXPR | LET_EXPR | IF_EXPR | SUM_EXPR
fn expr(parser: &mut Parser) {
    parser.assert_first(FIRST_EXPR);
    match parser.peek() {
        FUN => fun_expr(parser),
        LET => let_expr(parser),
        IF => if_expr(parser),
        _ => sum_expr(parser),
    }
}

// FUN_EXPR -> "fun" PARAM_LIST "->" EXPR
fn fun_expr(parser: &mut Parser) {
    parser.builder.start_node(FUN_EXPR.into());
    parser.consume(FUN);
    param_list(parser);
    if !parser.expect(ARROW, FIRST_EXPR, FOLLOW_EXPR, "FUN_EXPR") {
        parser.builder.finish_node();
        return;
    }
    expr(parser);
    parser.builder.finish_node();
}

// PARAM_LIST -> PARAM*
// PARAM -> ID_LOWER
fn param_list(parser: &mut Parser) {
    parser.builder.start_node(PARAM_LIST.into());
    while parser.peek() == ID_LOWER {
        parser.builder.start_node(PARAM.into());
        parser.consume(ID_LOWER);
        parser.builder.finish_node();
    }
    parser.builder.finish_node();
}

// LET_EXPR -> "let" LET_MOD LET_VAR "=" EXPR "in" EXPR
// LET_MOD -> "rec"?
fn let_expr(parser: &mut Parser) {
    parser.builder.start_node(LET_EXPR.into());
    parser.consume(LET);

    parser.builder.start_node(LET_MOD.into());
    if parser.peek() == REC {
        parser.consume(REC);
    }
    parser.builder.finish_node();
    let_var(parser);

    let mut token = parser.peek();
    if token != ASSIGN {
        parser.error(token, ASSIGN.as_set(), "LET_EXPR");
        parser.builder.start_node(ERROR.into());
        while token != ASSIGN && !FIRST_EXPR.contains(token) && !FOLLOW_EXPR.contains(token) {
            parser.consume(token);
            token = parser.peek();
        }
        parser.builder.finish_node();
        if token == ASSIGN {
            parser.consume(ASSIGN);
        } else if !FIRST_EXPR.contains(token) {
            parser.builder.finish_node();
            return;
        }
    } else {
        parser.consume(ASSIGN);
    }

    expr(parser);

    let mut token = parser.peek();
    if token != IN {
        parser.error(token, ASSIGN.as_set(), "LET_EXPR");
        parser.builder.start_node(ERROR.into());
        while token != ASSIGN && !FIRST_EXPR.contains(token) && !FOLLOW_EXPR.contains(token) {
            parser.consume(token);
            token = parser.peek();
        }
        parser.builder.finish_node();
        if token == ASSIGN {
            parser.consume(IN);
        } else if !FIRST_EXPR.contains(token) {
            parser.builder.finish_node();
            return;
        }
    } else {
        parser.consume(IN);
    }

    expr(parser);

    parser.builder.finish_node();
}

// LET_VAR -> ID_LOWER
fn let_var(parser: &mut Parser) {
    parser.builder.start_node(LET_VAR.into());
    let mut token = parser.peek();
    if token != ID_LOWER {
        parser.error(token, ID_LOWER.as_set(), "LET_VAR");
        while token != ID_LOWER && token != ASSIGN {
            parser.consume(token);
            token = parser.peek();
        }
        parser.builder.finish_node();
        if token != ID_LOWER {
            parser.builder.finish_node();
            return;
        }
    }
    parser.consume(ID_LOWER);
    parser.builder.finish_node();
}

// IF_EXPR -> "if" EXPR "then" EXPR "else" EXPR
fn if_expr(_: &mut Parser) {
    todo!()
}

// SUM_EXPR -> PROD_EXPR | SUM_EXPR SUM_OP PROD_EXPR
// SUM_OP -> "+" | "-"
fn sum_expr(parser: &mut Parser) {
    let checkpoint = parser.builder.checkpoint();
    prod_expr(parser);
    let mut token = parser.peek();
    while ADD_OPS.contains(token) {
        parser.builder.start_node_at(checkpoint, BINOP_EXPR.into());
        parser.builder.start_node(BINOP.into());
        parser.consume(token);
        parser.builder.finish_node();
        prod_expr(parser);
        parser.builder.finish_node();
        token = parser.peek();
    }
}

// PROD_EXPR -> ATOM_EXPR | PROD_EXPR PROD_OP ATOM_EXPR
// PROD_OP -> "*" | "/"
fn prod_expr(parser: &mut Parser) {
    let checkpoint = parser.builder.checkpoint();
    atom_expr(parser); // TODO(MH): This needs to be product_expr here and down.
    let mut token = parser.peek();
    while MUL_OPS.contains(token) {
        parser.builder.start_node_at(checkpoint, BINOP_EXPR.into());
        parser.builder.start_node(BINOP.into());
        parser.consume(token);
        parser.builder.finish_node();
        atom_expr(parser);
        parser.builder.finish_node();
        token = parser.peek();
    }
}

// fn app_expr(parser: &mut Parser) {
//     todo!()
// }

// ATOM_EXPR -> VAR_EXPR | LIT_EXPR | PAREN_EXPR
fn atom_expr(parser: &mut Parser) {
    let mut token = parser.peek();
    if !FIRST_ATOM_EXPR.contains(token) {
        parser.error(token, FIRST_ATOM_EXPR, "atom_expr");
        parser.builder.start_node(ERROR.into());
        while !FIRST_ATOM_EXPR.contains(token) && !FOLLOW_EXPR.contains(token) {
            parser.consume(token);
            token = parser.peek()
        }
        parser.builder.finish_node();
        if !FIRST_ATOM_EXPR.contains(token) {
            return;
        }
    }
    match parser.peek() {
        ID_LOWER => var_expr(parser),
        NAT_LIT | TRUE | FALSE => lit_expr(parser),
        LPAREN => paren_expr(parser),
        _ => unreachable!(),
    }
}

// VAR_EXPR -> ID_LOWER
fn var_expr(parser: &mut Parser) {
    parser.builder.start_node(VAR_EXPR.into());
    parser.consume(ID_LOWER);
    parser.builder.finish_node();
}

// LIT_EXPR -> LITERAL
// LITERAL -> NUM_LIT | TRUE | FALSE
fn lit_expr(parser: &mut Parser) {
    parser.builder.start_node(LIT_EXPR.into());
    parser.consume_in(LITERAL);
    parser.builder.finish_node();
}

// PAREN_EXPR -> "(" EXPR ")"
fn paren_expr(parser: &mut Parser) {
    parser.builder.start_node(PAREN_EXPR.into());
    parser.consume(LPAREN);
    expr(parser);
    let mut token = parser.peek();
    if token != RPAREN {
        parser.error(token, RPAREN.as_set(), "paren_expr");
        parser.builder.start_node(ERROR.into());
        while token != RPAREN && !FOLLOW_EXPR.contains(token) {
            parser.consume(token);
            token = parser.peek();
        }
        parser.builder.finish_node();
        if token != RPAREN {
            parser.builder.finish_node();
            return;
        }
    }
    parser.consume(RPAREN);
    parser.builder.finish_node();
}
