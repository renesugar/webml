use hir::*;
use pass::Pass;

pub struct FlatLet;

fn take_binds(mut expr: Expr) -> (Expr, Vec<Val>) {
    use hir::Expr::*;
    match expr {
        Binds { binds, ret, .. } => {
            expr = *ret;
            (expr, binds)
        }
        BinOp {
            ty,
            name,
            mut l,
            mut r,
        } => {
            let (l_, mut lbinds) = take_binds(*l);
            let (r_, mut rbinds) = take_binds(*r);
            l = Box::new(l_);
            r = Box::new(r_);
            lbinds.append(&mut rbinds);
            let expr = BinOp {
                ty: ty,
                name: name,
                l: l,
                r: r,
            };
            (expr, lbinds)
        }
        BuiltinCall { mut arg, ty, fun } => {
            let (a, mut binds) = take_binds(*arg);
            arg = Box::new(a);
            let expr = BuiltinCall {
                fun: fun,
                arg: arg,
                ty: ty,
            };
            (expr, binds)
        }
        App {
            mut fun,
            mut arg,
            ty,
        } => {
            let (f, mut fbinds) = take_binds(*fun);
            let (a, mut abinds) = take_binds(*arg);
            fun = Box::new(f);
            arg = Box::new(a);
            fbinds.append(&mut abinds);
            let expr = App {
                fun: fun,
                arg: arg,
                ty: ty,
            };
            (expr, fbinds)
        }
        If {
            mut cond,
            then,
            else_,
            ty,
        } => {
            let (c, cbinds) = take_binds(*cond);
            cond = Box::new(c);
            let expr = If {
                cond: cond,
                then: then,
                else_: else_,
                ty: ty,
            };
            (expr, cbinds)
        }
        Tuple { tys, tuple } => {
            let (tuple, bindss): (_, Vec<_>) = tuple.into_iter().map(take_binds).unzip();
            let expr = Tuple {
                tys: tys,
                tuple: tuple,
            };
            (expr, bindss.into_iter().flat_map(Vec::into_iter).collect())
        }
        x @ Fun { .. } | x @ Closure { .. } | x @ Sym { .. } | x @ Lit { .. } => (x, Vec::new()),
    }
}

impl FlatLet {
    pub fn new() -> Self {
        FlatLet
    }

    pub fn flat_hir(&mut self, mut hir: HIR) -> HIR {
        hir.0 = hir.0.into_iter().map(|val| self.flat_val(val)).collect();
        hir
    }

    fn flat_val(&mut self, mut val: Val) -> Val {
        val.expr = self.flat_expr(val.expr);
        val
    }

    fn flat_expr(&mut self, expr: Expr) -> Expr {
        use hir::Expr::*;
        match expr {
            Binds {
                mut binds,
                mut ret,
                ty,
            } => {
                let mut vec = Vec::new();
                for mut val in binds.into_iter() {
                    val.expr = self.flat_expr(val.expr);
                    let (expr, mut binds) = take_binds(val.expr);
                    val.expr = expr;
                    vec.append(&mut binds);
                    vec.push(val)
                }
                let ret_ = self.flat_expr(*ret);
                let (expr, mut binds_) = take_binds(ret_);
                ret = Box::new(expr);
                vec.append(&mut binds_);
                binds = vec;
                Binds {
                    binds: binds,
                    ret: ret,
                    ty: ty,
                }
            }
            BinOp {
                ty,
                name,
                mut l,
                mut r,
            } => {
                l = Box::new(self.flat_expr(*l));
                r = Box::new(self.flat_expr(*r));
                BinOp {
                    ty: ty,
                    name: name,
                    l: l,
                    r: r,
                }
            }
            Fun {
                mut body,
                param,
                body_ty,
                captures,
            } => {
                body = Box::new(self.flat_expr(*body));
                Fun {
                    body: body,
                    param: param,
                    body_ty: body_ty,
                    captures: captures,
                }
            }
            BuiltinCall { fun, mut arg, ty } => {
                arg = Box::new(self.flat_expr(*arg));
                BuiltinCall {
                    fun: fun,
                    arg: arg,
                    ty: ty,
                }
            }
            App {
                mut fun,
                mut arg,
                ty,
            } => {
                fun = Box::new(self.flat_expr(*fun));
                arg = Box::new(self.flat_expr(*arg));
                App {
                    fun: fun,
                    arg: arg,
                    ty: ty,
                }
            }
            If {
                mut cond,
                mut then,
                mut else_,
                ty,
            } => {
                cond = Box::new(self.flat_expr(*cond));
                then = Box::new(self.flat_expr(*then));
                else_ = Box::new(self.flat_expr(*else_));
                If {
                    ty: ty,
                    cond: cond,
                    then: then,
                    else_: else_,
                }
            }
            Tuple { tys, tuple } => {
                let tuple = tuple.into_iter().map(|t| self.flat_expr(t)).collect();
                Tuple {
                    tys: tys,
                    tuple: tuple,
                }
            }
            x @ Closure { .. } | x @ Sym { .. } | x @ Lit { .. } => x,
        }
    }
}

impl<E> Pass<HIR, E> for FlatLet {
    type Target = HIR;

    fn trans(&mut self, hir: HIR) -> ::std::result::Result<Self::Target, E> {
        Ok(self.flat_hir(hir))
    }
}
