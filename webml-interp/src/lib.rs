use std::path::Path;
use wasmtime::*;

fn add_ffi_module(linker: &mut Linker) {
    linker
        .func("js-ffi", "print", |x: i32| println!("{}", x))
        .expect("failed to add ffi functions");
}
fn add_rt_module(linker: &mut Linker) {
    let module_data =
        include_bytes!("../../webml-rt/target/wasm32-unknown-unknown/release/webml_rt.wasm");
    let module =
        Module::from_binary(linker.store(), module_data).expect("failed to compile webml_rt");
    let instance = linker
        .instantiate(&module)
        .expect("failed to instanciate webml_rt");
    linker
        .instance("webml-rt", &instance)
        .expect("failed to import webml-rt");
}

pub fn linker() -> Linker {
    let store = Store::default();
    let mut linker = Linker::new(&store);
    add_ffi_module(&mut linker);
    add_rt_module(&mut linker);
    linker
}

pub struct WebmlInterp {
    linker: Linker,
}

impl WebmlInterp {
    pub fn new() -> Self {
        Self { linker: linker() }
    }

    fn is_wasm(prog: &[u8]) -> bool {
        4 <= prog.len() && &prog[0..4] == b"\0asm"
    }

    pub fn run_file(&mut self, path: impl AsRef<Path>) {
        use std::fs;
        let prog = fs::read(path).expect("failed to read file");
        self.run(&prog)
    }

    pub fn run(&mut self, prog: &[u8]) {
        use std::str::from_utf8;
        if Self::is_wasm(prog) {
            self.run_wasm(prog)
        } else {
            let prog = from_utf8(prog).expect("program must be a utf-8 string");
            let module_data = Self::compile(prog);
            self.run_wasm(&module_data)
        }
    }

    pub fn run_wasm(&mut self, module_data: &[u8]) {
        let module = Module::from_binary(self.linker.store(), module_data)
            .expect("failed to compile module");
        self.linker
            .instantiate(&module)
            .expect("failed to instanciate module");
    }

    pub fn compile(input: &str) -> Vec<u8> {
        use webml::{compile_str, Config};
        let mut prelude = include_str!("../../ml_src/prelude.sml").to_string();
        prelude.push_str(input);
        compile_str(&prelude, &Config::default()).expect("failed to compile")
    }
}
