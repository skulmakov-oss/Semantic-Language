use std::fs;

#[test]
fn lowering_does_not_embed_crystalfold_logic() {
    let src = fs::read_to_string("crates/exo-ir/src/legacy_lowering.rs")
        .expect("read legacy_lowering.rs");

    assert!(
        src.contains("run_default_opt_passes"),
        "lowering pipeline must invoke IR opt passes"
    );
    assert!(
        !src.contains("fold_constants_and_identities"),
        "constant fold implementation must live in exo-ir passes"
    );
    assert!(
        !src.contains("enum ConstVal"),
        "const-fold state machine must not live in legacy_lowering"
    );
}

