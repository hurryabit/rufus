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

use crate::kind::{SyntaxExpecation, SyntaxKind::*, SyntaxKindSet};
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

pub fn root(parser: &mut Parser) {
    let mut parser = parser.with_node(ROOT);
    let mut parser = parser.with_follow(EOF.as_set());
    parser.expr();
    let mut token = parser.peek();
    if token != EOF {
        parser.error(token, EOF.as_set(), "root");
        let mut parser = parser.with_node(ERROR);
        while token != EOF {
            parser.consume(token);
            token = parser.peek();
        }
    }
}

impl<'a> Parser<'a> {
    fn expr(&mut self) {
        let parser = self;
        if !parser.find(FIRST_EXPR, "EXPR") {
            return;
        }
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
        parser.with_follow(ARROW).param_list();
        parser.with_follow(FIRST_EXPR).find_and_consume(ARROW, "FUN_EXPR");
        parser.expr()
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
            let mut parser = parser.with_follow(IN);
            {
                let mut parser = parser.with_node(LET_MOD);
                if parser.peek() == REC {
                    parser.consume(REC);
                }
            }
            parser.with_follow(ASSIGN).let_var();
            parser.with_follow(FIRST_EXPR).find_and_consume(ASSIGN, "LET_EXPR");
            parser.expr();
        }
        parser.with_follow(FIRST_EXPR).find_and_consume(IN, "LET_EXPR");
        parser.expr();
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
        let mut parser = self.with_follow(ADD_OPS);
        let checkpoint = parser.checkpoint();
        parser.prod_expr();
        while ADD_OPS.contains(parser.peek()) {
            let mut parser = parser.with_node_at(checkpoint, BINOP_EXPR);
            parser.with_node(BINOP).consume(ADD_OPS);
            parser.prod_expr();
        }
    }

    fn prod_expr(&mut self) {
        let mut parser = self.with_follow(MUL_OPS);
        let checkpoint = parser.checkpoint();
        parser.atom_expr();
        while MUL_OPS.contains(parser.peek()) {
            let mut parser = parser.with_node_at(checkpoint, BINOP_EXPR);
            parser.with_node(BINOP).consume(MUL_OPS);
            parser.atom_expr();
        }
    }

    fn atom_expr(&mut self) {
        let parser = self;
        if !parser.find(FIRST_ATOM_EXPR, "ATOM_EXPR") {
            return;
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
        self.with_node(LIT_EXPR).consume(LITERAL);
    }

    fn paren_expr(&mut self) {
        let mut parser = self.with_node(PAREN_EXPR);
        parser.consume(LPAREN);
        parser.with_follow(RPAREN).expr();
        parser.find_and_consume(RPAREN, "PAREN_EXPR");
    }
}
