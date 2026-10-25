#!/usr/bin/env python3
"""Writes the compiler test corpus (tests/compiler/cases/*.sm) and its expectation table
(tests/compiler/cases.json). The programs are hand-written Semantic source; expectations are
written by hand from the language rules, independently of both compilers. Run once after editing.

expect:
  ok                    compiles, admitted, runs to completion
  trap:<fragment>       compiles, admitted, VM run fails with a message containing <fragment>
  reject:<cat>:<frag>   the Semantic compiler rejects the source with category <cat>
"""
import json, pathlib

HERE = pathlib.Path(__file__).resolve().parent
CASES = {}

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
case("n08_break_outside_loop", "reject:context:'break' outside of a loop", """
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
case("n14_arith_on_bool", "reject:type:arithmetic expects i32 operands", """
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
case("n18_unsupported_record", "reject:unsupported:top-level item 'record'", """
record R {
    a: i32,
}

fn main() {
    return;
}
""")
case("n19_unsupported_text_literal", "reject:unsupported:text literals are outside the supported subset", """
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
case("n21_unsupported_for_loop", "reject:unsupported:'.' (field access, ranges", """
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
case("n24_unsupported_type_annotation", "reject:unsupported:only i32 and bool types", """
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
    }
}

fn main() {
    return;
}
""")
case("p16_if_else_both_return", "ok", """
fn sign(x: i32) -> i32 {
    if x < 0 {
        return -1;
    } else {
        return 1;
    }
}

fn main() {
    assert(sign(5) == 1);
    assert(sign(-3) == -1);
    return;
}
""")
case("n26_unexpected_character", "reject:syntax:unexpected character", """
fn main() {
    let x: i32 = 1 @ 2;
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
        source = CASES[name]["src"]
        # Preserve the approved return-path fixtures' exact bytes (no final LF).
        if name in ("n25_if_else_both_return_not_enough", "p16_if_else_both_return"):
            source = source.removesuffix("\n")
        (d / f"{name}.sm").write_bytes(source.encode("utf-8"))
        table[name] = CASES[name]["expect"]
    (HERE / "cases.json").write_text(json.dumps(table, indent=1, sort_keys=True) + "\n", encoding="utf-8")
    print(f"{len(table)} cases written")

if __name__ == "__main__":
    main()
