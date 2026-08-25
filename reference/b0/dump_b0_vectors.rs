// B0-00 reference extraction probe (Semantic-Language issue #2).
//
// This is NOT part of the reference repository (`skulmakov-oss/Semantic`)
// and is never written into it. It is copied here for reproducibility;
// `qualification/b0/check_reference_vectors.py` builds and runs it as the
// `src/main.rs` of a throwaway crate in its own temp directory, depending
// on the reference checkout's `crates/semantic-core-quad` via a `path`
// dependency - so the reference checkout is only ever read from.
//
// NOT/AND/OR/IMPLIES are computed by routing a single value through lane 0
// of a `QuadroReg32` and calling `.lattice_inverse()`/`.lattice_meet()`/
// `.lattice_join()` - the exact same call chain as `sm-vm`'s
// `quad_not`/`quad_and`/`quad_or`/`quad_implies`
// (`crates/sm-vm/src/semcode_vm.rs:3153-3168`, via its `quad_lane0`/
// `quad_lane0_value` helpers), not `QuadState`'s own (separately
// implemented) `inverse`/`meet`/`join` methods - so a divergence introduced
// only in the packed-register lattice path is exercised, not bypassed.
// Equality is `QuadState`'s own `PartialEq`, matching `sm-vm::value_eq`'s
// `Value::Quad(x) == Value::Quad(y)` (`x == y` on the bridged `QuadVal`).
use semantic_core_quad::{QuadState, QuadroReg32};

fn name(s: QuadState) -> &'static str {
    match s {
        QuadState::N => "N",
        QuadState::F => "F",
        QuadState::T => "T",
        QuadState::S => "S",
    }
}

fn quad_lane0(q: QuadState) -> QuadroReg32 {
    let mut reg = QuadroReg32::from_raw(0);
    reg.set_unchecked(0, q);
    reg
}

fn quad_lane0_value(reg: QuadroReg32) -> QuadState {
    reg.try_get(0).unwrap()
}

fn main() {
    println!("{{");
    println!("  \"crate\": \"semantic-core-quad\",");
    println!("  \"crate_version\": \"{}\",", env!("CARGO_PKG_VERSION"));
    println!("  \"generator\": \"reference/b0/dump_b0_vectors.rs\",");
    println!("  \"operation_family\": \"legacy_lattice\",");
    // Derived from `QuadState::bits()` itself (not hard-coded) so a future
    // discriminant change is caught by re-running this probe, not masked by
    // a stale literal that happens to still say 0/1/2/3.
    println!(
        "  \"state_encoding\": {{ \"N\": {}, \"F\": {}, \"T\": {}, \"S\": {} }},",
        QuadState::N.bits(),
        QuadState::F.bits(),
        QuadState::T.bits(),
        QuadState::S.bits()
    );

    println!("  \"not\": [");
    // Explicit canonical N/F/T/S order, deliberately NOT `QuadState::ALL`'s
    // own iteration order: this contract does not freeze that array's
    // internal ordering, only the four named states and their values, so a
    // pure reorder of `ALL` upstream must not appear as a corpus divergence
    // in this list-based (order-sensitive) output.
    let states = [QuadState::N, QuadState::F, QuadState::T, QuadState::S];
    for (i, a) in states.iter().enumerate() {
        let r = quad_lane0_value(quad_lane0(*a).lattice_inverse());
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

    dump_binary(
        "and",
        &|a, b| quad_lane0_value(quad_lane0(a).lattice_meet(quad_lane0(b))),
        false,
    );
    dump_binary(
        "or",
        &|a, b| quad_lane0_value(quad_lane0(a).lattice_join(quad_lane0(b))),
        false,
    );
    // IMPLIES per docs/spec/quad_logic_frame_v1.md: "A -> B = NOT(A) join B",
    // same call chain as `sm-vm::quad_implies`.
    dump_binary(
        "implies",
        &|a, b| {
            quad_lane0_value(
                quad_lane0(a)
                    .lattice_inverse()
                    .lattice_join(quad_lane0(b)),
            )
        },
        false,
    );

    println!("  \"eq\": [");
    let mut eq_entries = Vec::new();
    for a in states {
        for b in states {
            eq_entries.push((a, b, a == b));
        }
    }
    for (i, (a, b, r)) in eq_entries.iter().enumerate() {
        let comma = if i + 1 < eq_entries.len() { "," } else { "" };
        println!(
            "    {{ \"a\": \"{}\", \"b\": \"{}\", \"result\": {} }}{}",
            name(*a),
            name(*b),
            r,
            comma
        );
    }
    println!("  ]");

    println!("}}");
}
