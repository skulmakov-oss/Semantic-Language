// B0-00 reference extraction probe (Semantic-Language issue #2).
//
// This is NOT part of the reference repository (`skulmakov-oss/Semantic`)
// and is never written into it. It is copied here for reproducibility;
// `qualification/b0/check_reference_vectors.py` builds and runs it as the
// `src/main.rs` of a throwaway crate in its own temp directory, depending
// on the reference checkout's `semantic-core-quad`/`sm-format`/`sm-emit`/
// `sm-verify`/`sm-vm` crates via `path` dependencies - so the reference
// checkout is only ever read from.
//
// Four independent oracle surfaces, each exercised through its own real
// entry points rather than one one reimplemented by hand:
//
// - not/and/or/implies: lane 0 of a `QuadroReg32`, via
//   `.lattice_inverse()`/`.lattice_meet()`/`.lattice_join()` - the exact
//   call chain `sm-vm::quad_not`/`quad_and`/`quad_or`/`quad_implies` use
//   (`crates/sm-vm/src/semcode_vm.rs:3153-3168`), not `QuadState`'s own
//   separately-implemented single-value methods.
// - eq: `QuadState`'s own derived `PartialEq`.
// - vm_eq: the *source-language* `==` operator on quad literals, compiled
//   and run through the real public pipeline
//   (`sm_emit::compile_program_to_semcode` ->
//   `sm_verify::verify_semcode_token` ->
//   `sm_vm::run_verified_function_semcode_with_args`) - a genuinely
//   different code path from `eq` (the VM's `CmpEq` opcode, not
//   `QuadState::PartialEq`), since `sm-vm`'s own `value_eq` is a private
//   function with no direct external entry point.
// - opcode_encoding / minimum_semcode_revision: `sm_format::Opcode`'s own
//   public `.byte()` and `.minimum_semcode_revision()` methods for
//   QNot/QAnd/QOr/QImpl.
//
// The host-ABI boundary has two directions, both private functions with no
// direct external entry point:
// - inbound (quad_from_abi, byte -> Quad, including *rejection* of the 252
//   out-of-domain bytes) is proven by `check_reference_vectors.py` running
//   the reference's own exhaustive tests for it directly, since rejection
//   behavior across 256 inputs doesn't reduce to a simple JSON comparison.
// - outbound (quad_to_u8, Quad -> byte) IS mechanically extracted here
//   (`abi_encoding`), via the real host-call round trip: a hand-built
//   `IrInstr::GateWrite` program (the same low-level construction the
//   reference's own inbound tests use, since there is no `.sm` source
//   syntax for a host effect call), executed through the public
//   `run_verified_semcode_with_host_and_capabilities` with a real
//   `prom_abi::RecordingHostAbi`, reading back the exact byte it recorded.
use prom_abi::{AbiValue, RecordingHostAbi};
use prom_cap::{CapabilityKind, CapabilityManifest};
use semantic_core_quad::{QuadState, QuadroReg32};
use sm_ir::{IrFunction, IrInstr};
use sm_vm::Value;

// `IrInstr::LoadQ`'s `val` field is `sm_front::types::QuadVal` (the
// frontend/IR-level type), not `sm_vm::QuadVal` (the VM value-level type) -
// distinct types, per the B0-00 duplicate-owner audit.
fn quad_val(s: QuadState) -> sm_front::types::QuadVal {
    match s {
        QuadState::N => sm_front::types::QuadVal::N,
        QuadState::F => sm_front::types::QuadVal::F,
        QuadState::T => sm_front::types::QuadVal::T,
        QuadState::S => sm_front::types::QuadVal::S,
    }
}

/// Builds and runs a tiny program that loads `state` into a register and
/// writes it out via the real `GateWrite` host call, then reads back the
/// exact byte the host received - the real outbound host-ABI round trip,
/// not a re-derivation of `quad_to_u8`.
fn abi_outbound_byte(state: QuadState) -> u8 {
    let instrs = vec![
        IrInstr::LoadQ {
            dst: 0,
            val: quad_val(state),
        },
        IrInstr::GateWrite {
            device_id: 0,
            port: 0,
            src: 0,
        },
        IrInstr::Ret { src: None },
    ];
    let bytes = sm_ir::emit_ir_to_semcode(
        &[IrFunction {
            name: "main".to_string(),
            instrs,
            ownership_events: Vec::new(),
            params: Vec::new(),
        }],
        false,
    )
    .expect("emit abi_outbound_byte probe program");

    let mut host = RecordingHostAbi::default();
    let mut capabilities = CapabilityManifest::new();
    capabilities.allow(CapabilityKind::GateWrite);
    sm_vm::run_verified_semcode_with_host_and_capabilities(&bytes, &mut host, &capabilities)
        .expect("run abi_outbound_byte probe program");

    match host.writes.as_slice() {
        [(_, _, AbiValue::Quad(byte))] => *byte,
        other => panic!("expected exactly one recorded Quad gate_write, got {other:?}"),
    }
}

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

/// Compiles, verifies, and runs `{a} == {b}` as real Semantic source through
/// the public front-end/verify/VM pipeline, returning the VM's own boolean
/// result - exercises the `CmpEq` opcode path, not `QuadState::PartialEq`.
fn vm_eq(a: &str, b: &str) -> bool {
    let src = format!(
        "fn eq_case() -> bool {{ return {} == {}; }} fn main() {{ return; }}",
        a, b
    );
    let bytes = sm_emit::compile_program_to_semcode(&src).expect("compile eq_case");
    let token = sm_verify::verify_semcode_token(&bytes).expect("verify eq_case");
    let entry = token.require_entry("eq_case").expect("require_entry eq_case");
    match sm_vm::run_verified_function_semcode_with_args(&entry, Vec::new()).expect("run eq_case")
    {
        Value::Bool(b) => b,
        other => panic!("eq_case({a}, {b}) returned non-bool: {other:?}"),
    }
}

fn main() {
    println!("{{");
    println!("  \"crate\": \"semantic-core-quad\",");
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

    // Outbound host-ABI byte, via a real GateWrite round trip - see
    // abi_outbound_byte's doc comment.
    println!(
        "  \"abi_encoding\": {{ \"N\": {}, \"F\": {}, \"T\": {}, \"S\": {} }},",
        abi_outbound_byte(QuadState::N),
        abi_outbound_byte(QuadState::F),
        abi_outbound_byte(QuadState::T),
        abi_outbound_byte(QuadState::S)
    );

    // sm_format::Opcode's own public byte()/minimum_semcode_revision(), not
    // hand-copied literals - the frozen wire contract for the legacy
    // lattice family.
    use sm_format::semcode_format::Opcode;
    println!(
        "  \"opcode_encoding\": {{ \"QNot\": {}, \"QAnd\": {}, \"QOr\": {}, \"QImpl\": {} }},",
        Opcode::QNot.byte(),
        Opcode::QAnd.byte(),
        Opcode::QOr.byte(),
        Opcode::QImpl.byte()
    );
    println!(
        "  \"minimum_semcode_revision\": {{ \"QNot\": {}, \"QAnd\": {}, \"QOr\": {}, \"QImpl\": {} }},",
        Opcode::QNot.minimum_semcode_revision(),
        Opcode::QAnd.minimum_semcode_revision(),
        Opcode::QOr.minimum_semcode_revision(),
        Opcode::QImpl.minimum_semcode_revision()
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

    let dump_bool_binary = |field: &str, f: &dyn Fn(QuadState, QuadState) -> bool, last: bool| {
        println!("  \"{}\": [", field);
        let mut entries = Vec::new();
        for a in states {
            for b in states {
                entries.push((a, b, f(a, b)));
            }
        }
        for (i, (a, b, r)) in entries.iter().enumerate() {
            let comma = if i + 1 < entries.len() { "," } else { "" };
            println!(
                "    {{ \"a\": \"{}\", \"b\": \"{}\", \"result\": {} }}{}",
                name(*a),
                name(*b),
                r,
                comma
            );
        }
        println!("  ]{}", if last { "" } else { "," });
    };

    dump_bool_binary("eq", &|a, b| a == b, false);
    dump_bool_binary("vm_eq", &|a, b| vm_eq(name(a), name(b)), true);

    println!("}}");
}
