#!/usr/bin/env python3
"""Writes the compiler test corpus (tests/compiler/cases/*.sm) and its expectation table
(tests/compiler/cases.json). The programs are hand-written Semantic source; expectations are
written by hand from the language rules, independently of both compilers. Run once after editing.

Cases listed in POST_C0 use capabilities the pinned C0 (v1.2.0) lacks (text inspection
builtins, u32 arithmetic). They are compared with the exploratory reference only; C0's
outcome is recorded, never counted as a pass.

expect:
  ok                    compiles, admitted, runs to completion
  trap:<fragment>       compiles, admitted, VM run fails with a message containing <fragment>
  reject:<cat>:<frag>   the Semantic compiler rejects the source with category <cat>
"""
import json, pathlib

HERE = pathlib.Path(__file__).resolve().parent
CASES = {}
POST_C0 = set(['a06_text_ops', 'a07_u32_arithmetic_and_ordering', 'a09_option_match_text_bytes', 'a12_mini_lexer_workload', 't06_u32_underflow'])

def case(name, expect, src):
    CASES[name] = {"expect": expect, "src": src.strip("\n") + "\n"}

# ---------------------------------------------------------------- positive programs
case("p01_empty_main", "ok", """
fn main() {
    return;
}
""")
case("p02_two_functions", "ok", """
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn is_pos(x: i32) -> bool {
    return x > 0;
}

fn main() {
    let s: i32 = add(2, 3);
    assert(s == 5 && is_pos(s));
    return;
}
""")
case("p03_recursive_factorial", "ok", """
fn fact(n: i32) -> i32 {
    if n <= 1 {
        return 1;
    }
    return n * fact(n - 1);
}

fn main() {
    assert(fact(1) == 1);
    assert(fact(5) == 120);
    assert(fact(10) == 3628800);
    return;
}
""")
case("p04_mutual_recursion_call_before_definition", "ok", """
fn main() {
    assert(is_even(10));
    assert(!is_even(7));
    assert(is_odd(7));
    return;
}

fn is_even(n: i32) -> bool {
    if n == 0 {
        return true;
    }
    return is_odd(n - 1);
}

fn is_odd(n: i32) -> bool {
    if n == 0 {
        return false;
    }
    return is_even(n - 1);
}
""")
case("p05_fibonacci_loop_and_recursion", "ok", """
fn fib_rec(n: i32) -> i32 {
    if n < 2 {
        return n;
    }
    return fib_rec(n - 1) + fib_rec(n - 2);
}

fn fib_loop(n: i32) -> i32 {
    let mut a: i32 = 0;
    let mut b: i32 = 1;
    let mut i: i32 = 0;
    while i < n {
        let t: i32 = a + b;
        a = b;
        b = t;
        i = i + 1;
    }
    return a;
}

fn main() {
    assert(fib_rec(10) == 55);
    assert(fib_loop(10) == 55);
    assert(fib_loop(20) == 6765);
    return;
}
""")
case("p06_unit_function_statement_call", "ok", """
fn check(x: i32, expected: i32) {
    assert(x == expected);
}

fn main() {
    check(2 + 2, 4);
    check(10 / 3, 3);
    check(-7 % 3, -1);
    return;
}
""")
case("p07_break_continue", "ok", """
fn sum_odd_until(limit: i32, stop: i32) -> i32 {
    let mut i: i32 = 0;
    let mut s: i32 = 0;
    while i < limit {
        i = i + 1;
        if i % 2 == 0 {
            continue;
        }
        if i > stop {
            break;
        }
        s = s + i;
    }
    return s;
}

fn main() {
    assert(sum_odd_until(100, 9) == 25);
    assert(sum_odd_until(6, 100) == 9);
    return;
}
""")
case("p08_early_return_in_loop", "ok", """
fn first_multiple(of: i32, from: i32) -> i32 {
    let mut i: i32 = from;
    while true {
        if i % of == 0 {
            return i;
        }
        i = i + 1;
    }
    return -1;
}

fn main() {
    assert(first_multiple(7, 50) == 56);
    assert(first_multiple(5, 5) == 5);
    return;
}
""")
case("p09_parameter_reassignment_and_shadowing", "ok", """
fn countdown(n: i32) -> i32 {
    let mut steps: i32 = 0;
    while n > 0 {
        n = n - 1;
        steps = steps + 1;
    }
    let steps: i32 = steps * 10;
    return steps;
}

fn main() {
    assert(countdown(4) == 40);
    let x: i32 = 1;
    if x == 1 {
        let x: i32 = 2;
        assert(x == 2);
    }
    assert(x == 1);
    return;
}
""")
case("p10_nested_calls_and_bool_params", "ok", """
fn max(a: i32, b: i32) -> i32 {
    if a > b {
        return a;
    }
    return b;
}

fn pick(flag: bool, a: i32, b: i32) -> i32 {
    if flag {
        return a;
    }
    return b;
}

fn clamp(x: i32, lo: i32, hi: i32) -> i32 {
    return max(lo, -max(-x, -hi));
}

fn main() {
    assert(max(max(1, 9), max(4, 3)) == 9);
    assert(pick(3 < 2, 10, 20) == 20);
    assert(clamp(50, 0, 10) == 10);
    assert(clamp(-5, 0, 10) == 0);
    assert(clamp(7, 0, 10) == 7);
    return;
}
""")
case("p11_five_parameters_gcd", "ok", """
fn gcd(a: i32, b: i32) -> i32 {
    let mut x: i32 = a;
    let mut y: i32 = b;
    while y != 0 {
        let t: i32 = x % y;
        x = y;
        y = t;
    }
    return x;
}

fn weighted(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32 {
    return a * 1 + b * 2 + c * 3 + d * 4 + e * 5;
}

fn main() {
    assert(gcd(48, 18) == 6);
    assert(gcd(17, 5) == 1);
    assert(weighted(1, 1, 1, 1, 1) == 15);
    assert(weighted(5, 4, 3, 2, 1) == 35);
    return;
}
""")
case("p12_i32_wraparound_across_call", "ok", """
fn inc(x: i32) -> i32 {
    return x + 1;
}

fn main() {
    let big: i32 = 2147483647;
    assert(inc(big) == -2147483647 - 1);
    return;
}
""")
case("p13_nested_loops_with_calls", "ok", """
fn square(x: i32) -> i32 {
    return x * x;
}

fn main() {
    let mut total: i32 = 0;
    let mut i: i32 = 1;
    while i <= 4 {
        let mut j: i32 = 1;
        while j <= i {
            total = total + square(j);
            j = j + 1;
        }
        i = i + 1;
    }
    assert(total == 50);
    return;
}
""")
case("p14_else_if_chain_function", "ok", """
fn classify(n: i32) -> i32 {
    let mut r: i32 = 0;
    if n % 15 == 0 {
        r = 3;
    } else if n % 5 == 0 {
        r = 2;
    } else if n % 3 == 0 {
        r = 1;
    } else {
        r = 0;
    }
    return r;
}

fn main() {
    assert(classify(30) == 3);
    assert(classify(10) == 2);
    assert(classify(9) == 1);
    assert(classify(7) == 0);
    return;
}
""")
case("p15_discarded_value_call", "ok", """
fn noisy(x: i32) -> i32 {
    assert(x > 0);
    return x;
}

fn main() {
    noisy(5);
    return;
}
""")

# ---------------------------------------------------------------- runtime traps
case("t01_assert_in_callee", "trap:assertion failed", """
fn need_positive(x: i32) {
    assert(x > 0);
}

fn main() {
    need_positive(3);
    need_positive(-1);
    return;
}
""")
case("t02_division_by_zero_in_callee", "trap:DivisionByZero", """
fn div(a: i32, b: i32) -> i32 {
    return a / b;
}

fn main() {
    let z: i32 = div(10, 0);
    return;
}
""")
case("t03_min_div_minus_one", "trap:ArithmeticOverflow", """
fn div(a: i32, b: i32) -> i32 {
    return a / b;
}

fn main() {
    let m: i32 = -2147483647 - 1;
    let q: i32 = div(m, -1);
    return;
}
""")
case("t04_unbounded_recursion_frame_quota", "trap:quota", """
fn down(n: i32) -> i32 {
    return down(n + 1);
}

fn main() {
    let x: i32 = down(0);
    return;
}
""")

# ---------------------------------------------------------------- rejected sources
case("n01_unknown_function", "reject:name:unknown function 'nope'", """
fn main() {
    let x: i32 = nope(1);
    return;
}
""")
case("n02_wrong_arity", "reject:type:expects 2 argument(s), got 1", """
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn main() {
    let x: i32 = add(1);
    return;
}
""")
case("n03_wrong_argument_type", "reject:type:argument 1 of 'neg' expects i32, got bool", """
fn neg(a: i32) -> i32 {
    return -a;
}

fn main() {
    let x: i32 = neg(true);
    return;
}
""")
case("n04_return_type_mismatch", "reject:type:function returns i32 but this returns bool", """
fn f() -> i32 {
    return true;
}

fn main() {
    return;
}
""")
case("n05_missing_return", "reject:type:may exit without returning i32", """
fn f(x: i32) -> i32 {
    if x > 0 {
        return 1;
    }
}

fn main() {
    return;
}
""")
case("n06_value_return_in_unit_function", "reject:type:cannot return a value", """
fn f() {
    return 1;
}

fn main() {
    return;
}
""")
case("n07_empty_return_in_i32_function", "reject:type:'return;' in a function returning i32", """
fn f() -> i32 {
    return;
}

fn main() {
    return;
}
""")
case("n08_break_outside_loop", "reject:context:'break' is allowed only inside a while loop", """
fn main() {
    break;
}
""")
case("n09_duplicate_function", "reject:name:defined more than once", """
fn f() {
    return;
}

fn f() {
    return;
}

fn main() {
    return;
}
""")
case("n10_missing_main", "reject:name:missing entry function", """
fn helper() {
    return;
}
""")
case("n11_main_with_parameter", "reject:type:'main' must take no parameters", """
fn main(x: i32) {
    return;
}
""")
case("n12_unit_value_used", "reject:type:has unit type", """
fn f() {
    return;
}

fn main() {
    let x: i32 = f();
    return;
}
""")
case("n13_undefined_variable", "reject:name:undefined variable 'y'", """
fn main() {
    let x: i32 = y + 1;
    return;
}
""")
case("n14_arith_on_bool", "reject:type:arithmetic expects i32 or u32 operands", """
fn main() {
    assert(true + 1 == 2);
    return;
}
""")
case("n15_if_condition_i32", "reject:type:if condition expects bool, got i32", """
fn main() {
    if 1 {
        return;
    }
    return;
}
""")
case("n16_malformed_function", "reject:syntax:expected parameter name", """
fn main( {
    return;
}
""")
case("n17_invalid_expression", "reject:syntax:expected expression", """
fn main() {
    let x: i32 = 2 + * 3;
    return;
}
""")
case("p16_record_declaration_unused", "ok", """
record R {
    a: i32,
}

fn main() {
    return;
}
""")
case("p17_text_literal_binding", "ok", """
fn main() {
    let s: text = "hi";
    return;
}
""")
case("n20_unsupported_compound_assignment", "reject:unsupported:compound assignment", """
fn main() {
    let mut x: i32 = 1;
    x += 1;
    return;
}
""")
case("n21_unsupported_range_loop", "reject:unsupported:ranges are outside the supported subset", """
fn main() {
    for i in 0..3 {
        assert(true);
    }
    return;
}
""")
case("n22_literal_out_of_range", "reject:syntax:out of i32 range", """
fn main() {
    let x: i32 = 2147483648;
    return;
}
""")
case("n23_function_as_value", "reject:unsupported:function values", """
fn f() -> i32 {
    return 1;
}

fn main() {
    let g: i32 = f;
    return;
}
""")
case("n24_unsupported_type_annotation", "reject:unsupported:type 'quad'", """
fn f(q: quad) {
    return;
}

fn main() {
    return;
}
""")

case("n25_if_else_both_return_not_enough", "reject:type:may exit without returning i32", """
fn sign(x: i32) -> i32 {
    if x < 0 {
        return -1;
    } else {
        return 1;
    }
}

fn main() {
    return;
}
""")
case("n26_unexpected_character", "reject:syntax:unexpected character", """
fn main() {
    let x: i32 = 1 @ 2;
    return;
}
""")

# ---------------------------------------------------------------- aggregates, text, u32, match
case("a01_records_field_with", "ok", """
record Pt {
    x: i32,
    y: i32,
}

fn shift(p: Pt, d: i32) -> Pt {
    return p with { x: p.x + d };
}

fn main() {
    let a: Pt = Pt { y: 2, x: 1 };
    let b: Pt = shift(a, 10);
    assert(b.x == 11 && b.y == 2);
    assert(a.x == 1);
    return;
}
""")
case("a02_nested_records", "ok", """
record Inner {
    v: i32,
    ok: bool,
}

record Outer {
    name: text,
    inner: Inner,
}

fn main() {
    let o: Outer = Outer { name: "box", inner: Inner { v: 7, ok: true } };
    let o2: Outer = o with { inner: o.inner with { v: 8 } };
    assert(o.inner.v == 7 && o2.inner.v == 8 && o2.inner.ok && o2.name == "box");
    return;
}
""")
case("a03_tuples_and_destructuring", "ok", """
fn divmod(a: i32, b: i32) -> (i32, i32) {
    return (a / b, a % b);
}

fn main() {
    let (q, r) = divmod(17, 5);
    assert(q == 3 && r == 2);
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    (x, y) = divmod(9, 4);
    assert(x == 2 && y == 1);
    let t: (i32, bool, text) = (1, false, "z");
    let (n, f, s) = t;
    assert(n == 1 && !f && s == "z");
    return;
}
""")
case("a04_sequences_push_len_index_pop", "ok", """
fn squares(n: i32) -> Sequence(i32) {
    let mut out: Sequence(i32) = [];
    let mut i: i32 = 1;
    while i <= n {
        out = push(out, i * i);
        i = i + 1;
    }
    return out;
}

fn main() {
    let xs: Sequence(i32) = squares(5);
    assert(len(xs) == 5 && xs[0] == 1 && xs[4] == 25);
    let ys: Sequence(i32) = pop(xs);
    assert(len(ys) == 4 && len(xs) == 5);
    let lit: Sequence(bool) = [true, false, true];
    assert(lit[1] == false && len(lit) == 3);
    return;
}
""")
case("a05_for_in_and_while_break_continue", "ok", """
fn main() {
    let xs: Sequence(i32) = [3, 8, 1, 9, 4, 7];
    let mut sum: i32 = 0;
    let mut seen: i32 = 0;
    for x in xs {
        seen = seen + 1;
        if x % 2 == 1 {
            sum = sum + x;
        }
    }
    assert(sum == 20 && seen == 6);
    let mut i: i32 = 0;
    let mut odd: i32 = 0;
    while i < len(xs) {
        let x: i32 = xs[i];
        i = i + 1;
        if x % 2 == 0 {
            continue;
        }
        if x == 9 {
            break;
        }
        odd = odd + x;
    }
    assert(odd == 4);
    return;
}
""")
case("a06_text_ops", "ok", """
fn greet(name: text) -> text {
    return "hello, " + name;
}

fn main() {
    let g: text = greet("sm");
    assert(g == "hello, sm");
    assert(g != "hello");
    assert(text_len(g) == 9u32);
    assert(to_text(42) == "42" && to_text(true) == "true");
    return;
}
""")
case("a07_u32_arithmetic_and_ordering", "ok", """
fn sum_to(n: u32) -> u32 {
    let mut s: u32 = 0u32;
    let mut i: u32 = 1u32;
    while i <= n {
        s = s + i;
        i = i + 1u32;
    }
    return s;
}

fn main() {
    assert(sum_to(10u32) == 55u32);
    assert(4294967295u32 > 2147483647u32);
    assert(7u32 / 2u32 == 3u32 && 7u32 % 2u32 == 1u32);
    return;
}
""")
case("a08_match_i32_u32_value_and_statement", "ok", """
fn name_of(n: i32) -> text {
    return match n {
        1 => { "one" }
        2 => { "two" }
        _ => { "many" }
    };
}

fn main() {
    assert(name_of(1) == "one" && name_of(2) == "two" && name_of(9) == "many");
    let mut hits: i32 = 0;
    let b: u32 = 65u32;
    match b {
        65u32 => { hits = hits + 1; }
        _ => { hits = hits + 100; }
    }
    assert(hits == 1);
    return;
}
""")
case("a09_option_match_text_bytes", "ok", """
fn count_byte(s: text, target: u32) -> i32 {
    let mut i: u32 = 0u32;
    let mut n: i32 = 0;
    let len_s: u32 = text_len(s);
    while i < len_s {
        let b: u32 = match text_byte_at(s, i) {
            Option::Some(x) => { x }
            Option::None => { 0u32 }
        };
        if b == target {
            n = n + 1;
        }
        i = i + 1u32;
    }
    return n;
}

fn main() {
    assert(count_byte("banana", 97u32) == 3);
    let first: text = match text_slice("semantic", 0u32, 3u32) {
        Option::Some(t) => { t }
        Option::None => { "" }
    };
    assert(first == "sem");
    let none: bool = match text_byte_at("ab", 5u32) {
        Option::Some(x) => { false }
        Option::None => { true }
    };
    assert(none);
    return;
}
""")
case("a10_value_if_expression", "ok", """
fn abs(x: i32) -> i32 {
    let r: i32 = if x < 0 { -x } else { x };
    return r;
}

fn main() {
    assert(abs(-5) == 5 && abs(3) == 3);
    let label: text = if abs(-1) == 1 { "ok" } else { "bad" };
    assert(label == "ok");
    return;
}
""")
case("a11_sequence_of_records", "ok", """
record Tok {
    kind: i32,
    word: text,
}

fn tokens() -> Sequence(Tok) {
    let mut out: Sequence(Tok) = [];
    out = push(out, Tok { kind: 1, word: "let" });
    out = push(out, Tok { kind: 2, word: "x" });
    return out;
}

fn main() {
    let ts: Sequence(Tok) = tokens();
    let mut words: text = "";
    for t in ts {
        words = words + t.word + ";";
    }
    assert(words == "let;x;" && ts[1].kind == 2);
    return;
}
""")
case("a12_mini_lexer_workload", "ok", """
record Tok {
    kind: i32,
    start: u32,
    stop: u32,
}

fn lex(src: text) -> Sequence(Tok) {
    let n: u32 = text_len(src);
    let mut toks: Sequence(Tok) = [];
    let mut i: u32 = 0u32;
    while i < n {
        let b: u32 = match text_byte_at(src, i) {
            Option::Some(x) => { x }
            Option::None => { 0u32 }
        };
        if b == 32u32 {
            i = i + 1u32;
        } else if b >= 48u32 && b <= 57u32 {
            let start: u32 = i;
            let mut c: u32 = b;
            while c >= 48u32 && c <= 57u32 {
                i = i + 1u32;
                c = match text_byte_at(src, i) {
                    Option::Some(x) => { x }
                    Option::None => { 0u32 }
                };
            }
            toks = push(toks, Tok { kind: 1, start: start, stop: i });
        } else {
            toks = push(toks, Tok { kind: 2, start: i, stop: i + 1u32 });
            i = i + 1u32;
        }
    }
    return toks;
}

fn main() {
    let ts: Sequence(Tok) = lex("12 + 345 * 6");
    assert(len(ts) == 5);
    assert(ts[0].kind == 1 && ts[1].kind == 2 && ts[2].stop == 8u32);
    let w: text = match text_slice("12 + 345 * 6", ts[2].start, ts[2].stop) {
        Option::Some(t) => { t }
        Option::None => { "?" }
    };
    assert(w == "345");
    return;
}
""")
case("t05_index_out_of_bounds", "trap:out of bounds", """
fn main() {
    let xs: Sequence(i32) = [1, 2];
    let v: i32 = xs[5];
    return;
}
""")
case("t06_u32_underflow", "trap:ArithmeticOverflow", """
fn main() {
    let z: u32 = 0u32 - 1u32;
    return;
}
""")
case("n27_unknown_field", "reject:name:has no field 'z'", """
record Pt {
    x: i32,
}

fn main() {
    let p: Pt = Pt { x: 1 };
    let v: i32 = p.z;
    return;
}
""")
case("n28_missing_record_field", "reject:type:must give every field exactly once", """
record Pt {
    x: i32,
    y: i32,
}

fn main() {
    let p: Pt = Pt { x: 1 };
    return;
}
""")
case("n29_wrong_field_type", "reject:type:field 'x' expects i32, got bool", """
record Pt {
    x: i32,
}

fn main() {
    let p: Pt = Pt { x: true };
    return;
}
""")
case("n30_non_exhaustive_match", "reject:type:match is not exhaustive", """
fn main() {
    let n: i32 = 3;
    let v: i32 = match n {
        1 => { 10 }
    };
    return;
}
""")
case("n31_match_arm_types_differ", "reject:type:match arms have different types", """
fn main() {
    let n: i32 = 3;
    let v: i32 = match n {
        1 => { 10 }
        _ => { true }
    };
    return;
}
""")
case("n32_empty_sequence_without_type", "reject:type:element type of '[]' cannot be inferred", """
fn main() {
    let xs = [];
    return;
}
""")
case("n33_push_wrong_element", "reject:type:invalid arguments for builtin 'push'", """
fn main() {
    let xs: Sequence(i32) = [1];
    let ys: Sequence(i32) = push(xs, true);
    return;
}
""")
case("n34_mixed_u32_i32", "reject:type:of one family", """
fn main() {
    let a: u32 = 1u32;
    let b: i32 = 2;
    let c: u32 = a + b;
    return;
}
""")
case("n35_tuple_arity_mismatch", "reject:type:destructuring expects a 3-tuple", """
fn main() {
    let (a, b, c) = (1, 2);
    return;
}
""")
case("n36_record_equality_unsupported", "reject:unsupported:equality on", """
record Pt {
    x: i32,
}

fn main() {
    let a: Pt = Pt { x: 1 };
    let b: Pt = Pt { x: 1 };
    assert(a == b);
    return;
}
""")
case("n37_option_pattern_on_i32", "reject:type:match pattern does not fit", """
fn main() {
    let n: i32 = 1;
    let v: i32 = match n {
        Option::Some(x) => { x }
        _ => { 0 }
    };
    return;
}
""")
case("n38_u32_literal_out_of_range", "reject:syntax:out of u32 range", """
fn main() {
    let x: u32 = 4294967296u32;
    return;
}
""")

case("n39_continue_inside_for", "reject:context:'continue' is allowed only inside a while loop", """
fn main() {
    let xs: Sequence(i32) = [1, 2];
    for x in xs {
        continue;
    }
    return;
}
""")
case("n40_builtin_wrong_arity", "reject:type:expects 1 argument(s), got 0", """
fn main() {
    let n: i32 = len();
    return;
}
""")

case("a13_u32_match_unsuffixed_arms", "ok", """
fn digit(b: u32) -> i32 {
    return match b {
        48 => { 0 }
        49 => { 1 }
        57 => { 9 }
        _ => { 0 - 1 }
    };
}

fn main() {
    assert(digit(48u32) == 0 && digit(57u32) == 9 && digit(65u32) == -1);
    return;
}
""")

def main():
    d = HERE / "cases"
    d.mkdir(exist_ok=True)
    for old in d.glob("*.sm"):
        old.unlink()
    table = {}
    for name in sorted(CASES):
        (d / f"{name}.sm").write_bytes(CASES[name]["src"].encode("utf-8"))
        table[name] = {"expect": CASES[name]["expect"], "c0_comparable": name not in POST_C0}
    (HERE / "cases.json").write_text(json.dumps(table, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    unknown = POST_C0 - set(table)
    assert not unknown, unknown
    print(f"{len(table)} cases written")

if __name__ == "__main__":
    main()
