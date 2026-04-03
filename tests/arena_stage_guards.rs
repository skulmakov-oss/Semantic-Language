use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(root) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs_files(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

fn find_lines_with(root: &Path, needle: &str) -> Vec<(String, String)> {
    let mut files = Vec::new();
    collect_rs_files(root, &mut files);
    let mut hits = Vec::new();
    for file in files {
        let rel = file.to_string_lossy().replace('\\', "/");
        let txt = fs::read_to_string(&file).expect("read rs");
        for line in txt.lines() {
            let trimmed = line.trim();
            if trimmed.contains(needle) {
                hits.push((rel.clone(), trimmed.to_string()));
            }
        }
    }
    hits
}

#[test]
fn raw_expr_storage_is_confined_to_ast_arena() {
    let expr_vec_hits = find_lines_with(Path::new("crates/sm-front/src"), "Vec<Expr>");
    assert_eq!(
        expr_vec_hits,
        vec![(
            "crates/sm-front/src/types.rs".to_string(),
            "exprs: Vec<Expr>,".to_string()
        )],
        "raw Expr collections must remain owned only by AstArena"
    );

    let expr_box_hits = find_lines_with(Path::new("crates/sm-front/src"), "Box<Expr>");
    assert!(
        expr_box_hits.is_empty(),
        "frontend must not reintroduce boxed raw Expr ownership outside AstArena: {:?}",
        expr_box_hits
    );

    let expr_field_hits = find_lines_with(Path::new("crates/sm-front/src"), ": Expr,");
    assert!(
        expr_field_hits.is_empty(),
        "frontend fields must reference expressions by ExprId, not own Expr directly: {:?}",
        expr_field_hits
    );
}

#[test]
fn raw_stmt_storage_is_confined_to_ast_arena() {
    let stmt_vec_hits = find_lines_with(Path::new("crates/sm-front/src"), "Vec<Stmt>");
    assert_eq!(
        stmt_vec_hits,
        vec![(
            "crates/sm-front/src/types.rs".to_string(),
            "stmts: Vec<Stmt>,".to_string()
        )],
        "raw Stmt collections must remain owned only by AstArena"
    );

    let stmt_box_hits = find_lines_with(Path::new("crates/sm-front/src"), "Box<Stmt>");
    assert!(
        stmt_box_hits.is_empty(),
        "frontend must not reintroduce boxed raw Stmt ownership outside AstArena: {:?}",
        stmt_box_hits
    );

    let stmt_field_hits = find_lines_with(Path::new("crates/sm-front/src"), ": Stmt,");
    assert!(
        stmt_field_hits.is_empty(),
        "frontend fields must reference statements by StmtId, not own Stmt directly: {:?}",
        stmt_field_hits
    );
}

#[test]
fn crystalfold_stage_order_stays_cleanup_then_fold() {
    let passes =
        fs::read_to_string("crates/sm-ir/src/passes/mod.rs").expect("read passes mod");
    let cleanup_idx = passes
        .find("let cleanup = cleanup::StructuralCleanupPass;")
        .expect("cleanup stage exists");
    let fold_idx = passes
        .find("let fold = crystalfold::CrystalFoldPass::default();")
        .expect("crystalfold stage exists");
    assert!(
        cleanup_idx < fold_idx,
        "StructuralCleanup must remain ahead of CrystalFold in the default pass pipeline"
    );

    let opts = fs::read_to_string("docs/opts.md").expect("read opts doc");
    assert!(
        opts.contains("frontend -> semantics (warnings) -> lowering -> StructuralCleanup -> CrystalFold -> emit"),
        "docs/opts.md must keep the frozen CrystalFold pipeline contract"
    );
}
