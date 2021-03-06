use crate::ast::*;
use crate::util::{nspaces, PP};
use std::fmt;
use std::io;

impl<Ty: PP, DE: PP, DS: PP> PP for Context<Ty, DE, DS> {
    fn pp<W: io::Write>(&self, w: &mut W, indent: usize) -> io::Result<()> {
        self.1.pp(w, indent)
    }
}

impl<Ty: fmt::Display, DE: fmt::Display, DS: fmt::Display> fmt::Display for Context<Ty, DE, DS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = f.width().unwrap_or(0);
        write!(f, "{:indent$}", self.1, indent = indent)
    }
}

impl<Ty: PP, DE: PP, DS: PP> PP for AST<Ty, DE, DS> {
    fn pp<W: io::Write>(&self, w: &mut W, indent: usize) -> io::Result<()> {
        for bind in &self.0 {
            bind.pp(w, indent)?;
            write!(w, "\n")?;
        }
        Ok(())
    }
}

impl<Ty: fmt::Display, DE: fmt::Display, DS: fmt::Display> fmt::Display for AST<Ty, DE, DS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = f.width().unwrap_or(0);
        for bind in &self.0 {
            write!(f, "{:indent$}\n", bind, indent = indent)?;
        }
        Ok(())
    }
}

impl<Ty: PP, DE: PP, DS: PP> PP for Declaration<Ty, DE, DS> {
    fn pp<W: io::Write>(&self, w: &mut W, indent: usize) -> io::Result<()> {
        use Declaration::*;
        match self {
            Datatype { name, constructors } => {
                write!(w, "datatype ")?;
                name.pp(w, indent)?;
                write!(w, " =")?;
                inter_iter!(constructors, write!(w, " |")?, |(name, param)| =>{
                    write!(w, " ")?;
                    name.pp(w, indent)?;
                    if let Some(param) = param {
                        write!(w, " of ")?;
                        param.pp(w, indent)?;
                    }
                });
                Ok(())
            }
            Val { pattern, expr, rec } => {
                write!(w, "{}", Self::nspaces(indent))?;
                write!(w, "val ")?;
                if *rec {
                    write!(w, "rec ")?;
                }
                pattern.pp(w, indent)?;
                // write!(w, ": ")?;
                // self.ty.pp(w, indent)?;
                write!(w, " = ")?;
                expr.pp(w, indent + 4)?;
                Ok(())
            }
            D(d) => d.pp(w, indent),
        }
    }
}

impl<Ty: fmt::Display, DE: fmt::Display, DS: fmt::Display> fmt::Display
    for Declaration<Ty, DE, DS>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Declaration::*;
        let indent = f.width().unwrap_or(0);
        let next = indent + 4;

        match self {
            Datatype { name, constructors } => {
                write!(f, "datatype {:indent$} =", name, indent = indent)?;
                inter_iter!(constructors, write!(f, " |")?, |(name, param)| =>{
                    write!(f, " {:indent$}", name, indent = indent)?;
                    if let Some(param) = param {
                        write!(f, " of {:indent$}", param , indent = indent)?;
                    }
                });
                Ok(())
            }
            Val { pattern, expr, rec } => {
                write!(f, "{}val ", nspaces(indent))?;
                if *rec {
                    write!(f, "rec ")?;
                }
                write!(
                    f,
                    "{:indent$} = {:next$}",
                    pattern,
                    expr,
                    indent = indent,
                    next = next
                )?;
                Ok(())
            }
            D(d) => write!(f, "{:indent$}", d, indent = indent),
        }
    }
}

impl<Ty: PP> PP for DerivedDeclaration<Ty> {
    fn pp<W: io::Write>(&self, w: &mut W, indent: usize) -> io::Result<()> {
        use DerivedDeclaration::*;
        match self {
            Fun { name, clauses, .. } => {
                write!(w, "{}", Self::nspaces(indent))?;
                write!(w, "fun ")?;
                inter_iter!(
                    clauses,
                    { write!(w, "\n{}  | ", Self::nspaces(indent))? ; name.pp(w, indent)? },
                    |(params, expr)| => {
                    name.pp(w, indent)?;
                    write!(w, " ")?;
                    for param in params {
                        param.pp(w, indent)?;
                        write!(w, " ")?;
                    }
                    // write!(w, ": ")?;
                    // self.ty.pp(w, indent)?;
                    write!(w, " = ")?;
                    expr.pp(w, indent + 4)?;
                });
                Ok(())
            }
            Infix { priority, names } => {
                write!(w, "infix")?;
                if let Some(p) = priority {
                    write!(w, " {}", p)?;
                }
                for name in names {
                    write!(w, " ")?;
                    name.pp(w, indent)?;
                }
                Ok(())
            }
        }
    }
}

impl<Ty: fmt::Display> fmt::Display for DerivedDeclaration<Ty> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DerivedDeclaration::*;
        let indent = f.width().unwrap_or(0);
        let next = indent + 4;

        match self {
            Fun { name, clauses, .. } => {
                write!(f, "{}fun", nspaces(indent))?;
                inter_iter!(
                    clauses,
                     write!(f, "\n{}  | {}", nspaces(indent), name)?,
                    |(params, expr)| => {
                    write!(f, "{:indent$} ", name, indent = indent)?;
                    for param in params {
                        write!(f, "{:indent$} ", param, indent = indent)?;
                    }
                    // write!(w, ": ")?;
                    // self.ty.pp(w, indent)?;
                    write!(f, " = {:next$}", expr, next = next)?;
                });
                Ok(())
            }
            Infix { priority, names } => {
                write!(f, "infix")?;
                if let Some(p) = priority {
                    write!(f, " {}", p)?;
                }
                for name in names {
                    write!(f, " {}", name)?;
                }
                Ok(())
            }
        }
    }
}
impl<Ty: PP, DE: PP, DS: PP> PP for Expr<Ty, DE, DS> {
    fn pp<W: io::Write>(&self, w: &mut W, indent: usize) -> io::Result<()> {
        use crate::ast::ExprKind::*;
        match &self.inner {
            Binds { binds, ret } => {
                let ind = Self::nspaces(indent);
                let nextind = Self::nspaces(indent + 4);
                write!(w, "let\n")?;
                for val in binds {
                    val.pp(w, indent + 4)?;
                    write!(w, "\n")?;
                }
                write!(w, "{}in\n{}", ind, nextind)?;
                ret.pp(w, indent + 4)?;
                write!(w, "\n{}end", ind)?;
            }
            BuiltinCall { fun, args } => {
                write!(w, "_builtincall \"")?;
                fun.pp(w, indent)?;
                write!(w, "\"(")?;
                inter_iter! {
                    &args,
                    write!(w, ", ")?,
                    |arg| => {
                        arg.pp(w, indent)?
                    }
                }
                write!(w, ")")?;
            }
            ExternCall {
                module,
                fun,
                args,
                argty,
                retty,
            } => {
                write!(w, "_externcall \"{}\".\"{}\"(", module, fun)?;

                inter_iter! {
                    &args,
                    write!(w, ", ")?,
                    |arg| => {
                        arg.pp(w, indent)?
                    }
                };
                write!(w, "): (")?;
                inter_iter! {
                    &argty,
                    write!(w, ", ")?,
                    |ty| => {
                        ty.pp(w, indent)?
                    }
                };
                write!(w, ") -> ")?;
                retty.pp(w, indent)?;
            }
            Fn { body, param } => {
                write!(w, "fn ")?;
                param.pp(w, indent)?;
                write!(w, " => ")?;
                body.pp(w, indent + 4)?;
            }
            App { fun, arg } => {
                write!(w, "(")?;
                fun.pp(w, indent)?;
                write!(w, ") ")?;
                arg.pp(w, indent + 4)?;
            }
            Case { cond, clauses } => {
                let ind = Self::nspaces(indent);
                write!(w, "case ")?;
                cond.pp(w, indent + 4)?;
                write!(w, " of")?;
                for (pat, arm) in clauses {
                    write!(w, "\n{}", ind)?;
                    pat.pp(w, indent + 4)?;
                    write!(w, " => ")?;
                    arm.pp(w, indent + 4)?;
                }
            }
            Tuple { tuple } => {
                write!(w, "(")?;
                inter_iter! {
                    tuple.iter(),
                    write!(w, ", ")?,
                    |t| => {
                        t.pp(w, indent)?
                    }
                }
                write!(w, ")")?;
            }
            Symbol { name } => {
                name.pp(w, indent)?;
            }
            Constructor { name, arg } => {
                name.pp(w, indent)?;
                if let Some(arg) = arg {
                    write!(w, " ")?;
                    arg.pp(w, indent)?;
                }
            }
            Literal { value } => {
                value.pp(w, indent)?;
            }
            D(d) => {
                d.pp(w, indent)?;
            }
        }
        Ok(())
    }
}

impl<Ty: fmt::Display, DE: fmt::Display, DS: fmt::Display> fmt::Display for Expr<Ty, DE, DS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::ast::ExprKind::*;
        let indent = f.width().unwrap_or(0);
        let next = indent + 4;

        match &self.inner {
            Binds { binds, ret } => {
                let ind = nspaces(indent);
                let nextind = nspaces(next);
                write!(f, "let\n")?;
                for val in binds {
                    write!(f, "{:next$}\n", val, next = next)?;
                }
                write!(f, "{}in\n", ind)?;
                write!(f, "{}{:next$}", nextind, ret, next = next)?;
                write!(f, "\n{}end", ind)?;
            }
            BuiltinCall { fun, args } => {
                write!(f, "_builtincall \"{:indent$}\"(", fun, indent = indent)?;
                inter_iter! {
                    &args,
                    write!(f, ", ")?,
                    |arg| => {
                        write!(f,"{:indent$}", arg, indent = indent)?;
                    }
                }
                write!(f, ")")?;
            }
            ExternCall {
                module,
                fun,
                args,
                argty,
                retty,
            } => {
                write!(f, "_externcall \"{}\".\"{}\"(", module, fun)?;

                inter_iter! {
                    &args,
                    write!(f, ", ")?,
                    |arg| => {
                        write!(f,"{:indent$}", arg, indent = indent)?;
                    }
                };
                write!(f, "): (")?;
                inter_iter! {
                    &argty,
                    write!(f, ", ")?,
                    |ty| => {
                        write!(f,"{:indent$}", ty, indent = indent)?;
                    }
                };
                write!(f, ") -> {:indent$}", retty, indent = indent)?;
            }
            Fn { body, param } => {
                write!(
                    f,
                    "fn {:indent$} => {:next$}",
                    param,
                    body,
                    indent = indent,
                    next = next
                )?;
            }
            App { fun, arg } => {
                write!(
                    f,
                    "({:indent$}) {:next$}",
                    fun,
                    arg,
                    indent = indent,
                    next = next
                )?;
            }
            Case { cond, clauses } => {
                let ind = nspaces(indent);
                write!(f, "case {:next$} of", cond, next = next)?;
                for (pat, arm) in clauses {
                    write!(f, "\n{}{:next$}=>{:next$}", ind, pat, arm, next = next)?;
                }
            }
            Tuple { tuple } => {
                write!(f, "(")?;
                inter_iter! {
                    tuple.iter(),
                    write!(f, ", ")?,
                    |t| => {
                        write!(f,"{:indent$}", t, indent = indent)?;
                    }
                }
                write!(f, ")")?;
            }
            Symbol { name } => {
                write!(f, "{:indent$}", name, indent = indent)?;
            }
            Constructor { name, arg } => {
                write!(f, "{:indent$}", name, indent = indent)?;
                if let Some(arg) = arg {
                    write!(f, " {:indent$}", arg, indent = indent)?;
                }
            }
            Literal { value } => {
                write!(f, "{:indent$}", value, indent = indent)?;
            }
            D(d) => {
                write!(f, "{:indent$}", d, indent = indent)?;
            }
        }
        Ok(())
    }
}

impl<Ty: PP> PP for DerivedExprKind<Ty> {
    fn pp<W: io::Write>(&self, w: &mut W, indent: usize) -> io::Result<()> {
        use DerivedExprKind::*;
        match self {
            If {
                cond, then, else_, ..
            } => {
                let ind = nspaces(indent);
                write!(w, "if ")?;
                cond.pp(w, indent + 4)?;
                write!(w, "\n{}then ", ind)?;
                then.pp(w, indent + 4)?;
                write!(w, "\n{}else ", ind)?;
                else_.pp(w, indent + 4)?;
            }
        }
        Ok(())
    }
}

impl<Ty: fmt::Display> fmt::Display for DerivedExprKind<Ty> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DerivedExprKind::*;
        let indent = f.width().unwrap_or(0);
        let next = indent + 4;

        match self {
            If {
                cond, then, else_, ..
            } => {
                let ind = nspaces(indent);
                write!(f, "if {:next$}\n", cond, next = next)?;
                write!(f, "{}then {:next$}\n", ind, then, next = next)?;
                write!(f, "{}else {:next$}", ind, else_, next = next)?;
            }
        }
        Ok(())
    }
}

impl PP for Nothing {
    fn pp<W: io::Write>(&self, _: &mut W, _: usize) -> io::Result<()> {
        match *self {}
    }
}

impl fmt::Display for Nothing {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        match *self {}
    }
}

impl<Ty> PP for Pattern<Ty> {
    fn pp<W: io::Write>(&self, w: &mut W, indent: usize) -> io::Result<()> {
        use PatternKind::*;
        match &self.inner {
            Constant { value, .. } => write!(w, "{}", value),
            Char { value } => write!(w, r##"#"{}""##, value),
            Constructor { name, arg, .. } => {
                name.pp(w, indent)?;
                if let Some(arg) = arg {
                    // TODO: handle cases when its in function args
                    write!(w, " ")?;
                    arg.pp(w, indent)?;
                }

                Ok(())
            }
            Tuple { tuple, .. } => {
                write!(w, "(")?;
                inter_iter! {
                    tuple.iter(),
                    write!(w, ", ")?,
                    |pat| => {
                        pat.pp(w, indent)?
                    }
                }
                write!(w, ")")
            }
            Variable { name, .. } => name.pp(w, indent),
            Wildcard { .. } => write!(w, "_"),
        }
    }
}

impl<Ty> fmt::Display for Pattern<Ty> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use PatternKind::*;
        let indent = f.width().unwrap_or(0);

        match &self.inner {
            Constant { value, .. } => write!(f, "{}", value),
            Char { value } => write!(f, r##"#"{}""##, value),
            Constructor { name, arg, .. } => {
                write!(f, "{}", name)?;
                if let Some(arg) = arg {
                    // TODO: handle cases when its in function args
                    write!(f, " {:indent$}", arg, indent = indent)?;
                }

                Ok(())
            }
            Tuple { tuple, .. } => {
                write!(f, "(")?;
                inter_iter! {
                    tuple.iter(),
                    write!(f, ", ")?,
                    |pat| => {
                        write!(f, "{:indent$}", pat, indent = indent)?;
                    }
                }
                write!(f, ")")
            }
            Variable { name, .. } => write!(f, "{:indent$}", name, indent = indent),
            Wildcard { .. } => write!(f, "_"),
        }
    }
}

impl PP for Type {
    fn pp<W: io::Write>(&self, w: &mut W, indent: usize) -> io::Result<()> {
        use self::Type::*;
        match self {
            Variable(id) => write!(w, "'{}", id)?,
            Char => write!(w, "char")?,
            Int => write!(w, "int")?,
            Real => write!(w, "float")?,
            Fun(t1, t2) => {
                t1.pp(w, indent)?;
                write!(w, " -> ")?;
                t2.pp(w, indent)?;
            }
            Tuple(tys) => {
                write!(w, "(")?;
                for ty in tys.iter() {
                    ty.pp(w, indent)?;
                    write!(w, ", ")?;
                }
                write!(w, ")")?;
            }
            Datatype(name) => name.pp(w, indent)?,
        }
        Ok(())
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Type::*;
        match self {
            Variable(id) => write!(f, "'{}", id)?,
            Char => write!(f, "char")?,
            Int => write!(f, "int")?,
            Real => write!(f, "float")?,
            Fun(t1, t2) => {
                write!(f, "{} -> {}", t1, t2)?;
            }
            Tuple(tys) => {
                write!(f, "(")?;
                for ty in tys.iter() {
                    write!(f, "{}", ty)?;
                    write!(f, ", ")?;
                }
                write!(f, ")")?;
            }
            Datatype(name) => write!(f, "{}", name)?,
        }
        Ok(())
    }
}

impl PP for Empty {
    fn pp<W: io::Write>(&self, _: &mut W, _: usize) -> io::Result<()> {
        Ok(())
    }
}

impl fmt::Display for Empty {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}
