// use super::*;

// impl Expr {
//     pub fn children_mut(&mut self) -> impl Iterator<Item = &mut Expr> {
//         use genawaiter::{rc::gen, yield_};
//         use Expr::*;
//         gen!({
//             match self {
//                 Var(..) | Num(_) | Bool(_) | PrimOp(_) => {}
//                 App(f, es) => {
//                     yield_!(f.as_mut());
//                     for e in es {
//                         yield_!(e);
//                     }
//                 }
//                 Lam(_, e) | Proj(e, _) => {
//                     yield_!(e.as_mut());
//                 }
//                 Let(_, e1, e2) => {
//                     yield_!(e1.as_mut());
//                     yield_!(e2.as_mut());
//                 }
//                 If(e1, e2, e3) => {
//                     yield_!(e1.as_mut());
//                     yield_!(e2.as_mut());
//                     yield_!(e3.as_mut());
//                 }
//                 Record(_, es) => {
//                     for e in es {
//                         yield_!(e);
//                     }
//                 }
//             }
//         })
//         .into_iter()
//     }
// }
