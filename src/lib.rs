#[macro_use]
pub mod util;
pub mod ast;
pub mod backend;
mod config;
pub mod hir;
pub mod id;
pub mod lir;
pub mod mir;
mod parser;
pub mod pass;
pub mod prim;
mod unification_pool;

pub use crate::ast::TypeError;
pub use crate::config::Config;
pub use crate::parser::parse;
pub use crate::pass::{Chain, Pass};

pub fn compile_str<'a>(input: &'a str, config: &Config) -> Result<Vec<u8>, TypeError<'a>> {
    use crate::pass::{ConvError, PrintablePass};
    use wasm::Dump;

    let id = id::Id::new();

    let mut passes = compile_pass![
       parse: ConvError::new(parse),
       desugar: ast::Desugar::new(id.clone()),
       rename: ast::Rename::new(id.clone()),
       var_to_constructor: ast::VarToConstructor::new(id.clone()),
       typing: ast::Typer::new(),
       case_simplify: ast::CaseSimplify::new(id.clone()),
       ast_to_hir: hir::AST2HIR::new(id.clone()),
       constructor_to_enum: hir::ConstructorToEnum::new(),
       simplify: hir::Simplify::new(id.clone()),
       flattening_expression: hir::FlatExpr::new(id.clone()),
       flattening_let: hir::FlatLet::new(),
       unnest_functions: hir::UnnestFunc::new(id.clone()),
       closure_conversion: hir::ForceClosure::new(),
       hir_to_mir: mir::HIR2MIR::new(id),
       unalias: mir::UnAlias::new(),
       block_arrange: mir::BlockArrange::new(),
       mir_to_lir: lir::MIR2LIR::new(),
       backend: backend::LIR2WASM::new(),
    ];

    let module: backend::Output = passes.trans(input, config)?;

    let mut code = Vec::new();
    module.0.dump(&mut code);
    Ok(code)
}
