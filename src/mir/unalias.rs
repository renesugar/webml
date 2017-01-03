use std::collections::HashMap;

use mir::*;
use prim::*;
use pass::Pass;



pub struct UnAlias {
    alias: HashMap<Symbol, Symbol>,
}


impl UnAlias {
    pub fn new() -> Self {
        UnAlias { alias: HashMap::new() }
    }

    fn conv_mir(&mut self, mir: MIR) -> MIR {
        MIR(mir.0.into_iter().map(|f| self.conv_fun(f)).collect())
    }

    fn conv_fun(&mut self, mut fun: Function) -> Function {
        self.alias.clear();
        let mut body = Vec::new();
        for mut ebb in fun.body.into_iter() {
            ebb = self.conv_ebb(ebb);
            body.push(ebb);
        }
        fun.body = body;
        fun
    }

    fn conv_ebb(&mut self, mut ebb: EBB) -> EBB {
        use mir::Op::*;
        let mut body = Vec::new();
        for mut op in ebb.body.into_iter() {
            match &mut op {
                &mut Alias { ref var, ref sym, .. } => {
                    self.alias(var.clone(), sym.clone());
                    continue;
                }
                &mut Add { ref mut l, ref mut r, .. } |
                &mut Mul { ref mut l, ref mut r, .. } => {
                    self.resolv_alias(l);
                    self.resolv_alias(r);
                }
                &mut Closure { ref mut fun, ref mut env, .. } => {
                    self.resolv_alias(fun);
                    for &mut (_, ref mut var) in env.iter_mut() {
                        self.resolv_alias(var);
                    }
                }
                &mut Call { ref mut fun, ref mut args, .. } => {
                    self.resolv_alias(fun);
                    for arg in args.iter_mut() {
                        self.resolv_alias(arg);
                    }
                }
                &mut Jump { ref mut args, .. } => {
                    for arg in args.iter_mut() {
                        self.resolv_alias(arg);
                    }
                }
                &mut Ret { ref mut value, .. } => self.resolv_alias(value),
                &mut Lit { .. } => (),
                &mut Branch { ref mut cond, .. } => self.resolv_alias(cond),
            }
            body.push(op)
        }
        ebb.body = body;
        ebb
    }

    fn alias(&mut self, al: Symbol, mut orig: Symbol) {
        while let Some(s) = self.alias.get(&orig) {
            orig = s.clone()
        }
        self.alias.insert(al, orig);
    }


    fn resolv_alias(&mut self, sym: &mut Symbol) {
        match self.alias.get(sym) {
            None => (),
            Some(orig) => sym.0 = orig.0.clone(),
        }
    }
}

impl Pass<MIR> for UnAlias {
    type Target = MIR;
    type Err = TypeError;

    fn trans(&mut self, mir: MIR) -> ::std::result::Result<Self::Target, Self::Err> {
        Ok(self.conv_mir(mir))
    }
}