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
    parser.expr();
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

impl<'a> Parser<'a> {
    fn expr(&mut self) {
        let parser = self;
        parser.assert_first(FIRST_EXPR);
        match parser.peek() {
            FUN => parser.fun_expr(),
            LET => parser.let_expr(),
            IF => parser.if_expr(),
            _ => parser.sum_expr(),
        }
    }

    fn fun_expr(&mut self) {
        let mut parser = self.with_node(FUN_EXPR);
        parser.consume(FUN);
        parser.param_list();
        if !parser.find_before(ARROW, FIRST_EXPR, FOLLOW_EXPR, "FUN_EXPR") {
            return;
        }
        parser.expr();
    }

    fn param_list(&mut self) {
        let mut parser = self.with_node(PARAM_LIST);
        while parser.peek() == ID_LOWER {
            let mut parser = parser.with_node(PARAM);
            parser.consume(ID_LOWER);
        }
    }

    fn let_expr(&mut self) {
        let mut parser = self.with_node(LET_EXPR);
        parser.consume(LET);
        {
            let mut parser = parser.with_node(LET_MOD);
            if parser.peek() == REC {
                parser.consume(REC);
            }
        }
        parser.let_var();
        if parser.find_before(ASSIGN, FIRST_EXPR, FOLLOW_EXPR, "LET_EXPR") {
            parser.expr();
        }
        if parser.find_before(IN, FIRST_EXPR, FOLLOW_EXPR, "LET_EXPR") {
            parser.expr();
        }
    }

    fn let_var(&mut self) {
        let mut parser = self.with_node(LET_VAR);
        let mut token = parser.peek();
        if token != ID_LOWER {
            parser.error(token, ID_LOWER.as_set(), "LET_VAR");
            while token != ID_LOWER && token != ASSIGN {
                parser.consume(token);
                token = parser.peek();
            }
            if token != ID_LOWER {
                return;
            }
        }
        parser.consume(ID_LOWER);
    }

    fn if_expr(&mut self) {
        todo!()
    }

    fn sum_expr(&mut self) {
        let parser = self;
        let checkpoint = parser.builder.checkpoint();
        parser.prod_expr();
        let mut token = parser.peek();
        while ADD_OPS.contains(token) {
            parser.builder.start_node_at(checkpoint, BINOP_EXPR.into());
            parser.builder.start_node(BINOP.into());
            parser.consume(token);
            parser.builder.finish_node();
            parser.prod_expr();
            parser.builder.finish_node();
            token = parser.peek();
        }
    }

    fn prod_expr(&mut self) {
        let parser = self;
        let checkpoint = parser.builder.checkpoint();
        parser.atom_expr();
        let mut token = parser.peek();
        while MUL_OPS.contains(token) {
            parser.builder.start_node_at(checkpoint, BINOP_EXPR.into());
            parser.builder.start_node(BINOP.into());
            parser.consume(token);
            parser.builder.finish_node();
            parser.atom_expr();
            parser.builder.finish_node();
            token = parser.peek();
        }
    }

    fn atom_expr(&mut self) {
        let parser = self;
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
            ID_LOWER => parser.var_expr(),
            NAT_LIT | TRUE | FALSE => parser.lit_expr(),
            LPAREN => parser.paren_expr(),
            _ => unreachable!(),
        }
    }

    fn var_expr(&mut self) {
        self.with_node(VAR_EXPR).consume(ID_LOWER);
    }

    fn lit_expr(&mut self) {
        self.with_node(LIT_EXPR).consume_in(LITERAL);
    }

    fn paren_expr(&mut self) {
        let parser = self;
        parser.builder.start_node(PAREN_EXPR.into());
        parser.consume(LPAREN);
        parser.expr();
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
}
