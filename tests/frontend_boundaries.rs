use semantic_language::frontend;

#[test]
fn boundary_core_ir_emit_are_accessible() {
    let src = "fn main() { return; }";

    let p = frontend::core::parse_program(src).expect("parse");
    let fns = frontend::core::build_fn_table(&p).expect("fn table");
    let _ = frontend::ir::lower_function_to_ir(&p.functions[0], &p.arena, &fns).expect("lower");

    let bytes = frontend::emit::compile_program_to_semcode(src).expect("emit");
    assert!(bytes.len() >= 8);
}
