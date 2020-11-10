use super::*;

impl Module {
    pub fn type_decls(&self) -> impl Iterator<Item = &TypeDecl> {
        self.decls.iter().filter_map(|decl| match decl {
            Decl::Type(type_decl) => Some(type_decl),
            Decl::Func(_) => None,
        })
    }

    pub fn type_decls_mut(&mut self) -> impl Iterator<Item = &mut TypeDecl> {
        self.decls.iter_mut().filter_map(|decl| match decl {
            Decl::Type(type_decl) => Some(type_decl),
            Decl::Func(_) => None,
        })
    }

    pub fn func_decls(&self) -> impl Iterator<Item = &FuncDecl> {
        self.decls.iter().filter_map(|decl| match decl {
            Decl::Func(func) => Some(func),
            Decl::Type(_) => None,
        })
    }

    pub fn func_decls_mut(&mut self) -> impl Iterator<Item = &mut FuncDecl> {
        self.decls.iter_mut().filter_map(|decl| match decl {
            Decl::Func(func) => Some(func),
            Decl::Type(_) => None,
        })
    }
}

impl Type {
    pub fn children_mut(&mut self) -> impl Iterator<Item = &mut Type> {
        use genawaiter::{rc::gen, yield_};
        use Type::*;
        gen!({
            match self {
                Error => {}
                Var(_) | Syn(_) | Int | Bool => {}
                Fun(params, result) => {
                    for param in params {
                        yield_!(param);
                    }
                    yield_!(result);
                }
                App(fun, args) => {
                    yield_!(fun);
                    for arg in args {
                        yield_!(arg);
                    }
                }
                Record(fields) => {
                    for (_name, typ) in fields {
                        yield_!(typ);
                    }
                }
                Variant(constrs) => {
                    for (_name, opt_typ) in constrs {
                        if let Some(typ) = opt_typ {
                            yield_!(typ);
                        }
                    }
                }
            }
        })
        .into_iter()
    }
}

impl Expr {
    pub fn children_mut(&mut self) -> impl Iterator<Item = &mut Expr> {
        use genawaiter::{rc::gen, yield_};
        use Expr::*;
        gen!({
            match self {
                Error => {}
                Var(_) | Num(_) | Bool(_) => {}
                Lam(_params, body) => {
                    yield_!(body.as_mut());
                }
                App(fun, args) => {
                    yield_!(fun);
                    for arg in args {
                        yield_!(arg);
                    }
                }
                BinOp(lhs, _opcode, rhs) => {
                    yield_!(lhs);
                    yield_!(rhs);
                }
                TypeAbs(params, body) => {
                    yield_!(body);
                }
                TypeApp(fun, _args) => {
                    yield_!(fun);
                }
                Let(_binder, _type, bindee, body) => {
                    yield_!(bindee);
                    yield_!(body);
                }
                If(cond, then, elze) => {
                    yield_!(cond);
                    yield_!(then);
                    yield_!(elze);
                }
                Record(fields) => {
                    for (_name, expr) in fields {
                        yield_!(expr);
                    }
                }
                Proj(record, _field) => {
                    yield_!(record);
                }
                Variant(_const, opt_payload) => {
                    if let Some(payload) = opt_payload {
                        yield_!(payload);
                    }
                }
                Match(scrut, branches) => {
                    yield_!(scrut);
                    for branch in branches {
                        yield_!(&mut branch.rhs);
                    }
                }
            }
        })
        .into_iter()
    }
}
