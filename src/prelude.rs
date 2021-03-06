use std::sync::Arc;
use std::collections::HashMap;

use ast;
use intrinsics;
use Module;
use Type;

/// Argument lifetime constraint.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Lt {
    Arg(usize),
    Return,
    Default,
}

/// Stores preloaded function constraints.
/// These are already checked.
#[derive(Clone, PartialEq, Debug)]
pub struct Dfn {
    pub lts: Vec<Lt>,
    pub tys: Vec<Type>,
    pub ret: Type,
}

impl Dfn {
    pub fn new(f: &ast::Function) -> Dfn {
        let mut lts: Vec<Lt> = vec![];
        let mut tys: Vec<Type> = vec![];
        'next_arg: for arg in &f.args {
            if let Some(ref lt) = arg.lifetime {
                if **lt == "return" {
                    lts.push(Lt::Return);
                    continue 'next_arg;
                }
                for (i, arg2) in f.args.iter().enumerate() {
                    if **arg2.name == **lt {
                        lts.push(Lt::Arg(i));
                        continue 'next_arg;
                    }
                }
                panic!("Could not find argument `{}`", lt);
            } else {
                lts.push(Lt::Default);
            }
            tys.push(arg.ty.clone());
        }
        Dfn {
            lts: lts,
            tys: tys,
            ret: f.ret.clone(),
        }
    }

    pub fn returns(&self) -> bool { self.ret != Type::Void }
}

pub struct Prelude {
    pub functions: HashMap<Arc<String>, usize>,
    pub list: Vec<Dfn>,
}

impl Prelude {
    pub fn insert(&mut self, name: Arc<String>, f: Dfn) {
        let n = self.list.len();
        self.functions.insert(name, n);
        self.list.push(f);
    }

    pub fn intrinsic(&mut self, name: Arc<String>, index: usize, f: Dfn) {
        let n = self.list.len();
        assert!(n == index, "{}", name);
        self.functions.insert(name, n);
        self.list.push(f);
    }

    pub fn new() -> Prelude {
        Prelude {
            functions: HashMap::new(),
            list: vec![]
        }
    }

    pub fn new_intrinsics() -> Prelude {
        let mut prelude = Prelude::new();
        intrinsics::standard(&mut prelude);
        prelude
    }

    pub fn from_module(module: &Module) -> Prelude {
        let mut prelude = Prelude::new();
        intrinsics::standard(&mut prelude);
        for f in &*module.ext_prelude {
            prelude.insert(f.name.clone(), f.p.clone());
        }
        for f in &module.functions {
            prelude.insert(f.name.clone(), Dfn::new(f));
        }
        prelude
    }
}
