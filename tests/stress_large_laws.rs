use semantic_language::frontend::parse_logos_program;
use semantic_language::semantics::analyze_logos_program;

fn build_large_logos(law_count: usize, whens_per_law: usize) -> String {
    let mut src = String::new();
    src.push_str("Entity Sensor:\n");
    src.push_str("    state val: quad\n");
    src.push_str("    prop active: bool\n\n");
    for i in 0..law_count {
        let prio = (i % 100) + 1;
        src.push_str(&format!("Law \"Law{}\" [priority {}]:\n", i, prio));
        for _ in 0..whens_per_law {
            src.push_str("    When true -> System.recovery()\n");
        }
        src.push('\n');
    }
    src
}

#[test]
fn stress_large_law_count_semantics_is_stable() {
    let src = build_large_logos(1200, 2);
    let parsed = parse_logos_program(&src).expect("parse large logos");
    let report_a = analyze_logos_program(&parsed, &src).expect("analyze A");
    let report_b = analyze_logos_program(&parsed, &src).expect("analyze B");

    assert_eq!(report_a.scheduled_laws.len(), 1200);
    assert_eq!(report_a.scheduled_laws, report_b.scheduled_laws);
}
