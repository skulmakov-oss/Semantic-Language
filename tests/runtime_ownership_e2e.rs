use sm_emit::compile_program_to_semcode;
use sm_ir::semcode_format::{
    read_u16_le, read_u32_le, read_u8, read_utf8, MAGIC11, OWNERSHIP_EVENT_KIND_BORROW,
    OWNERSHIP_EVENT_KIND_WRITE, OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX, OWNERSHIP_SECTION_TAG,
};
use sm_verify::verify_semcode;
use sm_vm::{run_verified_semcode, RuntimeError};

#[derive(Clone, Copy)]
struct OwnershipEventSpec<'a> {
    kind: u8,
    root: &'a str,
    components: &'a [u16],
}

struct FunctionLayout {
    strings: Vec<String>,
    ownership_start: Option<usize>,
    instr_start: usize,
}

const DETERMINISTIC_RUNS: usize = 8;

#[test]
fn runtime_ownership_sibling_write_passes_on_verified_path() {
    let bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    assert_eq!(&bytes[..8], &MAGIC11);

    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[0],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[1],
            },
        ],
    );

    verify_semcode(&rewritten).expect("verify");
    run_verified_semcode(&rewritten).expect("sibling tuple write should pass");
}

#[test]
fn runtime_ownership_rejects_same_path_write_deterministically() {
    let bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[0],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[0],
            },
        ],
    );

    assert_write_overlap_rejects_deterministically(&rewritten, "pair");
}

#[test]
fn runtime_ownership_rejects_parent_child_overlap_deterministically() {
    let bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[0],
            },
        ],
    );

    assert_write_overlap_rejects_deterministically(&rewritten, "pair");
}

#[test]
fn runtime_ownership_rejects_child_parent_overlap_deterministically() {
    let bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[0],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[],
            },
        ],
    );

    assert_write_overlap_rejects_deterministically(&rewritten, "pair");
}

#[test]
fn runtime_ownership_inner_frame_borrow_does_not_leak_after_exit() {
    let bytes = compile_program_to_semcode(multi_frame_source()).expect("compile");
    assert_eq!(&bytes[..8], &MAGIC11);
    assert!(function_has_ownership_section(&bytes, "helper"));
    assert!(function_has_ownership_section(&bytes, "main"));

    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[OwnershipEventSpec {
            kind: OWNERSHIP_EVENT_KIND_WRITE,
            root: "pair",
            components: &[],
        }],
    );

    verify_semcode(&rewritten).expect("verify");
    run_verified_semcode(&rewritten).expect("inner-frame borrow must not leak after return");
}

#[test]
fn runtime_ownership_does_not_silently_claim_record_or_adt_support() {
    for src in [record_source(), adt_source()] {
        let bytes = compile_program_to_semcode(src).expect("compile");
        assert_ne!(&bytes[..8], &MAGIC11);
        assert!(!any_function_has_ownership_section(&bytes));
        verify_semcode(&bytes).expect("verify");
        run_verified_semcode(&bytes).expect("run");
    }
}

#[test]
fn runtime_ownership_sibling_write_is_stable_across_runs() {
    let bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[0],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[1],
            },
        ],
    );

    assert_repeated_verified_success(&rewritten, DETERMINISTIC_RUNS);
}

#[test]
fn runtime_ownership_same_path_rejects_identically_across_runs() {
    let bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[0],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[0],
            },
        ],
    );

    assert_repeated_write_overlap_rejects(&rewritten, "pair", DETERMINISTIC_RUNS);
}

#[test]
fn runtime_ownership_parent_child_rejects_identically_across_runs() {
    let bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[0],
            },
        ],
    );

    assert_repeated_write_overlap_rejects(&rewritten, "pair", DETERMINISTIC_RUNS);
}

#[test]
fn runtime_ownership_child_parent_rejects_identically_across_runs() {
    let bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[0],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[],
            },
        ],
    );

    assert_repeated_write_overlap_rejects(&rewritten, "pair", DETERMINISTIC_RUNS);
}

#[test]
fn runtime_ownership_multi_frame_cleanup_is_stable_across_runs() {
    let bytes = compile_program_to_semcode(multi_frame_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[OwnershipEventSpec {
            kind: OWNERSHIP_EVENT_KIND_WRITE,
            root: "pair",
            components: &[],
        }],
    );

    assert_repeated_verified_success(&rewritten, DETERMINISTIC_RUNS);
}

fn tuple_assignment_source() -> &'static str {
    r#"
        fn main() {
            let pair: (i32, bool) = (1, true);
            let other: i32 = 0;
            (pair, other) = ((2, false), 1);
            return;
        }
    "#
}

fn multi_frame_source() -> &'static str {
    r#"
        fn helper(pair: (i32, bool)) {
            let (ref left, _): (i32, bool) = pair;
            let _ = left;
            return;
        }

        fn main() {
            let pair: (i32, bool) = (1, true);
            let other: i32 = 0;
            helper((3, false));
            (pair, other) = ((2, false), 1);
            return;
        }
    "#
}

fn record_source() -> &'static str {
    r#"
        record DecisionContext {
            camera: quad,
            quality: f64,
        }

        fn main() {
            let ctx: DecisionContext = DecisionContext { quality: 0.75, camera: T };
            let shadow: DecisionContext = ctx;
            let _ = shadow;
            return;
        }
    "#
}

fn adt_source() -> &'static str {
    r#"
        enum Maybe {
            None,
            Some(bool),
        }

        fn choose(flag: bool) -> Maybe {
            return Maybe::Some(flag);
        }

        fn main() {
            let left: Maybe = choose(true);
            let right: Maybe = Maybe::None;
            let _ = left;
            let _ = right;
            return;
        }
    "#
}

fn assert_write_overlap_rejects_deterministically(bytes: &[u8], symbol_name: &str) {
    assert_repeated_write_overlap_rejects(bytes, symbol_name, 2);
}

fn assert_repeated_verified_success(bytes: &[u8], runs: usize) {
    verify_semcode(bytes).expect("verify");
    for _ in 0..runs {
        run_verified_semcode(bytes).expect("verified run must stay successful");
    }
}

fn assert_repeated_write_overlap_rejects(bytes: &[u8], symbol_name: &str, runs: usize) {
    verify_semcode(bytes).expect("verify");

    let mut observed = Vec::with_capacity(runs);
    for _ in 0..runs {
        let err = run_verified_semcode(bytes).expect_err("runtime overlap must reject");
        let rendered = format!("{err}");
        assert!(matches!(
            err,
            RuntimeError::TypeMismatchRuntime(message)
                if message == format!("write path overlaps active borrow for '{symbol_name}'")
        ));
        observed.push(rendered);
    }

    for rendered in &observed[1..] {
        assert_eq!(rendered, &observed[0]);
    }
}

fn any_function_has_ownership_section(bytes: &[u8]) -> bool {
    let mut cursor = 8usize;
    while cursor < bytes.len() {
        let (name, code, next) = next_function(bytes, cursor);
        let _ = name;
        if parse_function_layout(code).ownership_start.is_some() {
            return true;
        }
        cursor = next;
    }
    false
}

fn function_has_ownership_section(bytes: &[u8], target: &str) -> bool {
    let (_, code, _) = find_function(bytes, target);
    parse_function_layout(code).ownership_start.is_some()
}

fn rewrite_function_ownership_events(
    bytes: &[u8],
    target: &str,
    events: &[OwnershipEventSpec<'_>],
) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    out.extend_from_slice(&bytes[..8]);

    let mut cursor = 8usize;
    let mut rewrote = false;
    while cursor < bytes.len() {
        let (name, code, next) = next_function(bytes, cursor);
        let rewritten = if name == target {
            rewrote = true;
            rewrite_function_code(code, events)
        } else {
            code.to_vec()
        };

        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(&(rewritten.len() as u32).to_le_bytes());
        out.extend_from_slice(&rewritten);
        cursor = next;
    }

    assert!(rewrote, "target function '{target}' not found");
    out
}

fn rewrite_function_code(code: &[u8], events: &[OwnershipEventSpec<'_>]) -> Vec<u8> {
    let layout = parse_function_layout(code);
    let ownership_start = layout.ownership_start.expect("OWN0 section");
    let mut out = Vec::with_capacity(code.len());
    out.extend_from_slice(&code[..ownership_start]);
    out.extend_from_slice(&ownership_section_bytes(&layout.strings, events));
    out.extend_from_slice(&code[layout.instr_start..]);
    out
}

fn ownership_section_bytes(strings: &[String], events: &[OwnershipEventSpec<'_>]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&OWNERSHIP_SECTION_TAG);
    out.extend_from_slice(&(events.len() as u16).to_le_bytes());
    for event in events {
        let root = strings
            .iter()
            .position(|name| name == event.root)
            .unwrap_or_else(|| panic!("missing root '{}'", event.root)) as u32;
        append_ownership_event(&mut out, event.kind, root, event.components);
    }
    out
}

fn append_ownership_event(out: &mut Vec<u8>, kind: u8, root: u32, components: &[u16]) {
    out.push(kind);
    out.extend_from_slice(&root.to_le_bytes());
    out.extend_from_slice(&(components.len() as u16).to_le_bytes());
    for index in components {
        out.push(OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX);
        out.extend_from_slice(&index.to_le_bytes());
    }
}

fn parse_function_layout(code: &[u8]) -> FunctionLayout {
    let mut cursor = 0usize;
    let string_count = read_u16_le(code, &mut cursor).expect("string count") as usize;
    let mut strings = Vec::with_capacity(string_count);
    for _ in 0..string_count {
        let len = read_u16_le(code, &mut cursor).expect("string len") as usize;
        strings.push(
            read_utf8(code, &mut cursor, len)
                .expect("string")
                .to_string(),
        );
    }

    if cursor + 4 <= code.len() && &code[cursor..cursor + 4] == b"DBG0" {
        cursor += 4;
        let count = read_u16_le(code, &mut cursor).expect("debug count") as usize;
        for _ in 0..count {
            let _ = read_u32_le(code, &mut cursor).expect("debug pc");
            let _ = read_u32_le(code, &mut cursor).expect("debug line");
            let _ = read_u16_le(code, &mut cursor).expect("debug col");
        }
    }

    let ownership_start = if cursor + 4 <= code.len() && &code[cursor..cursor + 4] == OWNERSHIP_SECTION_TAG
    {
        Some(cursor)
    } else {
        None
    };

    if ownership_start.is_some() {
        cursor += OWNERSHIP_SECTION_TAG.len();
        let count = read_u16_le(code, &mut cursor).expect("ownership count") as usize;
        for _ in 0..count {
            let _ = read_u8(code, &mut cursor).expect("ownership kind");
            let _ = read_u32_le(code, &mut cursor).expect("ownership root");
            let component_count =
                read_u16_le(code, &mut cursor).expect("ownership component count") as usize;
            for _ in 0..component_count {
                let kind = read_u8(code, &mut cursor).expect("ownership component kind");
                assert_eq!(kind, OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX);
                let _ = read_u16_le(code, &mut cursor).expect("ownership component value");
            }
        }
    }

    FunctionLayout {
        strings,
        ownership_start,
        instr_start: cursor,
    }
}

fn find_function<'a>(bytes: &'a [u8], target: &str) -> (String, &'a [u8], usize) {
    let mut cursor = 8usize;
    while cursor < bytes.len() {
        let (name, code, next) = next_function(bytes, cursor);
        if name == target {
            return (name, code, next);
        }
        cursor = next;
    }
    panic!("function '{target}' not found");
}

fn next_function<'a>(bytes: &'a [u8], start: usize) -> (String, &'a [u8], usize) {
    let mut cursor = start;
    let name_len = read_u16_le(bytes, &mut cursor).expect("function name len") as usize;
    let name = read_utf8(bytes, &mut cursor, name_len).expect("function name");
    let code_len = read_u32_le(bytes, &mut cursor).expect("function code len") as usize;
    let code_start = cursor;
    let code_end = code_start + code_len;
    (name, &bytes[code_start..code_end], code_end)
}
