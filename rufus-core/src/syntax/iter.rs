use super::*;

pub struct ChildrenMut<'a> {
    expr1: Option<&'a mut Expr>,
    expr2: Option<&'a mut Expr>,
    expr3: Option<&'a mut Expr>,
    exprs: std::slice::IterMut<'a, Expr>,
}

impl<'a> ChildrenMut<'a> {
    fn new(
        expr1: Option<&'a mut Expr>,
        expr2: Option<&'a mut Expr>,
        expr3: Option<&'a mut Expr>,
        exprs: std::slice::IterMut<'a, Expr>,
    ) -> Self {
        ChildrenMut {
            expr1,
            expr2,
            expr3,
            exprs,
        }
    }
}

impl<'a> Iterator for ChildrenMut<'a> {
    type Item = &'a mut Expr;

    fn next(&mut self) -> Option<Self::Item> {
        self.expr1
            .take()
            .or_else(|| self.expr2.take())
            .or_else(|| self.expr3.take())
            .or_else(|| self.exprs.next())
    }
}

impl Expr {
    pub fn children_mut(&mut self) -> impl Iterator<Item = &mut Expr> {
        use Expr::*;
        match self {
            Var(..) | Num(_) | Bool(_) | PrimOp(_) => {
                ChildrenMut::new(None, None, None, [].iter_mut())
            }
            App(f, es) => ChildrenMut::new(Some(f), None, None, es.iter_mut()),
            Lam(_, e) | Proj(e, _) => ChildrenMut::new(Some(e), None, None, [].iter_mut()),
            Let(_, e1, e2) => ChildrenMut::new(Some(e1), Some(e2), None, [].iter_mut()),
            If(e1, e2, e3) => ChildrenMut::new(Some(e1), Some(e2), Some(e3), [].iter_mut()),
            Record(_, es) => ChildrenMut::new(None, None, None, es.iter_mut()),
        }
    }
}
