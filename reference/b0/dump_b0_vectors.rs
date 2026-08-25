// B0-00 reference extraction probe (Semantic-Language issue #2).
//
// This is NOT part of the reference repository (`skulmakov-oss/Semantic`).
// It is copied here for reproducibility and is placed into
// `crates/semantic-core-quad/examples/` of a checked-out reference repo by
// `qualification/b0/check_reference_vectors.py`, run there with
// `cargo run -p semantic-core-quad --example dump_b0_vectors`, and removed
// afterward. It must never be committed to the reference repository itself.
//
// It dumps the exhaustive NOT/AND/OR/IMPLIES legacy-lattice truth tables
// straight from `QuadState`'s own `inverse`/`meet`/`join` methods (the same
// primitives `sm-vm::quad_not`/`quad_and`/`quad_or`/`quad_implies` delegate
// to via `QuadroReg32`), so the corpus is derived from the reference
// implementation, not retyped by hand.
use semantic_core_quad::QuadState;

fn name(s: QuadState) -> &'static str {
    match s {
        QuadState::N => "N",
        QuadState::F => "F",
        QuadState::T => "T",
        QuadState::S => "S",
    }
}

fn main() {
    println!("{{");
    println!("  \"crate\": \"semantic-core-quad\",");
    println!("  \"crate_version\": \"{}\",", env!("CARGO_PKG_VERSION"));
    println!("  \"generator\": \"crates/semantic-core-quad/examples/dump_b0_vectors.rs\",");
    println!("  \"operation_family\": \"legacy_lattice\",");
    println!("  \"state_encoding\": {{ \"N\": 0, \"F\": 1, \"T\": 2, \"S\": 3 }},");

    println!("  \"not\": [");
    let states = QuadState::ALL;
    for (i, a) in states.iter().enumerate() {
        let r = a.inverse();
        let comma = if i + 1 < states.len() { "," } else { "" };
        println!(
            "    {{ \"a\": \"{}\", \"result\": \"{}\" }}{}",
            name(*a),
            name(r),
            comma
        );
    }
    println!("  ],");

    let dump_binary = |op_name: &str, f: &dyn Fn(QuadState, QuadState) -> QuadState, last: bool| {
        println!("  \"{}\": [", op_name);
        let mut entries = Vec::new();
        for a in states {
            for b in states {
                entries.push((a, b, f(a, b)));
            }
        }
        for (i, (a, b, r)) in entries.iter().enumerate() {
            let comma = if i + 1 < entries.len() { "," } else { "" };
            println!(
                "    {{ \"a\": \"{}\", \"b\": \"{}\", \"result\": \"{}\" }}{}",
                name(*a),
                name(*b),
                name(*r),
                comma
            );
        }
        println!("  ]{}", if last { "" } else { "," });
    };

    dump_binary("and", &|a, b| a.meet(b), false);
    dump_binary("or", &|a, b| a.join(b), false);
    // IMPLIES per docs/spec/quad_logic_frame_v1.md: "A -> B = NOT(A) join B".
    dump_binary("implies", &|a, b| a.inverse().join(b), true);

    println!("}}");
}
