#!/usr/bin/env python3
"""Writes the Semantic VM compatibility corpus: tests/vm/corpus/<case>.sm + corpus.json.

Each case is an ordinary Semantic program, compiled by the reference compiler; the
artifact (not the source) is what both VMs execute. `expect` is the outcome the case is
written to produce on the reference VM:
  ok                      completes
  error:<message prefix>  fails with that runtime message
`semvm` overrides the expectation for the Semantic VM where it deliberately differs:
  unsupported             the VM must fail explicitly with "unsupported: ..."
"""
import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent
CASES = {}


def case(name, expect, src, semvm=None):
    CASES[name] = {"expect": expect, "semvm": semvm, "src": src.strip() + "\n"}


# ---------------------------------------------------------------- basic execution
case("b01_empty_main", "ok", """
fn main() {
    return;
}
""")

case("b02_i32_arithmetic", "ok", """
fn main() {
    let a: i32 = 17;
    let b: i32 = 5;
    print(to_text(a + b) + " " + to_text(a - b) + " " + to_text(a * b) + " " + to_text(a / b) + " " + to_text(a % b));
    let n: i32 = 0 - 7;
    print(to_text(n / 2) + " " + to_text(n % 2) + " " + to_text(7 / (0 - 2)) + " " + to_text(7 % (0 - 2)));
    let big: i32 = 2147483647;
    print(to_text(big + 1) + " " + to_text(big * 2) + " " + to_text((0 - big) - 2));
    return;
}
""")

case("b03_bool_logic", "ok", """
fn main() {
    let t: bool = true;
    let f: bool = false;
    print(to_text(t && f) + " " + to_text(t || f) + " " + to_text(!t) + " " + to_text(f == f) + " " + to_text(t != f));
    print(to_text(3 < 4) + " " + to_text(4 <= 4) + " " + to_text(5 > 4) + " " + to_text(4 >= 5) + " " + to_text(0 - 1 < 0));
    return;
}
""")

case("b04_branching", "ok", """
fn classify(n: i32) -> text {
    let mut r: text = "";
    if n < 0 {
        r = "negative";
    } else if n == 0 {
        r = "zero";
    } else if n < 10 {
        r = "small";
    } else {
        r = "large";
    }
    return r;
}

fn main() {
    print(classify(0 - 3) + " " + classify(0) + " " + classify(7) + " " + classify(100));
    return;
}
""")

case("b05_loops", "ok", """
fn main() {
    let mut sum: i32 = 0;
    let mut i: i32 = 1;
    while i <= 100 {
        sum = sum + i;
        i = i + 1;
    }
    let mut odd: i32 = 0;
    let mut j: i32 = 0;
    while true {
        j = j + 1;
        if j > 20 {
            break;
        }
        if j % 2 == 0 {
            continue;
        }
        odd = odd + j;
    }
    let mut grid: i32 = 0;
    let mut r: i32 = 0;
    while r < 5 {
        let mut c: i32 = 0;
        while c < 5 {
            grid = grid + r * c;
            c = c + 1;
        }
        r = r + 1;
    }
    print(to_text(sum) + " " + to_text(odd) + " " + to_text(grid));
    return;
}
""")

case("b06_assert_pass", "ok", """
fn main() {
    assert(1 + 1 == 2);
    assert(true);
    print("asserted");
    return;
}
""")

# ---------------------------------------------------------------- functions
case("f01_parameters", "ok", """
fn describe(n: i32, flag: bool, label: text, big: u32) -> text {
    return label + ":" + to_text(n) + ":" + to_text(flag) + ":" + to_text(big);
}

fn main() {
    print(describe(42, true, "x", 4000000000u32));
    print(describe(0 - 1, false, "", 0u32));
    return;
}
""")

case("f02_nested_calls", "ok", """
fn inc(x: i32) -> i32 {
    return x + 1;
}

fn twice(x: i32) -> i32 {
    return inc(inc(x));
}

fn quad4(x: i32) -> i32 {
    return twice(twice(x));
}

fn main() {
    print(to_text(quad4(10)) + " " + to_text(inc(twice(quad4(0)))));
    return;
}
""")

case("f03_recursion", "ok", """
fn fact(n: i32) -> i32 {
    if n <= 1 {
        return 1;
    }
    return n * fact(n - 1);
}

fn sum_to(n: i32) -> i32 {
    if n == 0 {
        return 0;
    }
    return n + sum_to(n - 1);
}

fn main() {
    print(to_text(fact(10)) + " " + to_text(sum_to(200)));
    return;
}
""")

case("f04_mutual_recursion", "ok", """
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

fn main() {
    print(to_text(is_even(10)) + " " + to_text(is_odd(7)) + " " + to_text(is_even(99)));
    return;
}
""")

case("f05_fibonacci", "ok", """
fn fib(n: i32) -> i32 {
    if n < 2 {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

fn main() {
    print(to_text(fib(15)));
    return;
}
""")

case("f06_unit_function", "ok", """
fn shout(t: text) {
    print(t + "!");
    return;
}

fn main() {
    shout("a");
    shout("b");
    return;
}
""")

# ---------------------------------------------------------------- values
case("v01_text", "ok", """
fn main() {
    let a: text = "Sem";
    let b: text = "antic";
    let c: text = a + b;
    print(c + " " + to_text(c == "Semantic") + " " + to_text(c != a) + " " + to_text(text_len(c)));
    let first: u32 = match text_byte_at(c, 0u32) {
        Option::Some(x) => { x }
        Option::None => { 0u32 }
    };
    let past: bool = match text_byte_at(c, 99u32) {
        Option::Some(x) => { false }
        Option::None => { true }
    };
    let mid: text = match text_slice(c, 1u32, 4u32) {
        Option::Some(s) => { s }
        Option::None => { "?" }
    };
    print(to_text(first) + " " + to_text(past) + " " + mid);
    print(to_text(text_starts_with(c, "Se")) + " " + to_text(text_ends_with(c, "x")) + " " + to_text(text_is_empty("")));
    return;
}
""")

case("v02_sequences", "ok", """
fn main() {
    let mut xs: Sequence(i32) = [5, 3, 8];
    let mut i: i32 = 0;
    while i < 5 {
        xs = push(xs, i * i);
        i = i + 1;
    }
    let mut total: i32 = 0;
    for x in xs {
        total = total + x;
    }
    xs = prepend(xs, 100);
    let ys: Sequence(i32) = pop(xs);
    print(to_text(len(xs)) + " " + to_text(xs[0]) + " " + to_text(xs[8]) + " " + to_text(total) + " " + to_text(len(ys)));
    print(to_text(contains(xs, 8)) + " " + to_text(contains(xs, 7)) + " " + to_text(is_empty(ys)) + " " + to_text(ys == [100, 5, 3, 8, 0, 1, 4, 9]));
    let words: Sequence(text) = ["a", "bb", "ccc"];
    let mut joined: text = "";
    for w in words {
        joined = joined + w + ",";
    }
    print(joined);
    return;
}
""")

case("v03_tuples", "ok", """
fn divmod(a: i32, b: i32) -> (i32, i32) {
    return (a / b, a % b);
}

fn main() {
    let p: (i32, (bool, text)) = (1, (true, "in"));
    print(to_text(divmod(9, 4) == (2, 1)) + " " + to_text(p == (1, (true, "out"))) + " " + to_text(p == (1, (true, "in"))));
    let (q, r) = divmod(17, 5);
    let (n, inner) = p;
    let (flag, word) = inner;
    print(to_text(q) + " " + to_text(r) + " " + to_text(n) + " " + to_text(flag) + " " + word);
    return;
}
""")

case("v04_records", "ok", """
record Point {
    x: i32,
    y: i32
}

record Box {
    lo: Point,
    hi: Point,
    name: text
}

fn area(b: Box) -> i32 {
    return (b.hi.x - b.lo.x) * (b.hi.y - b.lo.y);
}

fn main() {
    let a: Point = Point { x: 1, y: 2 };
    let b: Point = a with { y: 10 };
    let bx: Box = Box { lo: a, hi: Point { x: 4, y: 6 }, name: "box" };
    print(to_text(a.x) + " " + to_text(b.y) + " " + to_text(area(bx)) + " " + bx.name);
    print(to_text(a == Point { x: 1, y: 2 }) + " " + to_text(a == b) + " " + to_text(bx.lo == a));
    return;
}
""")

case("v05_option", "ok", """
fn find(xs: Sequence(i32), target: i32) -> Option(i32) {
    let mut i: i32 = 0;
    let mut found: Option(i32) = Option::None;
    for x in xs {
        if x == target {
            found = Option::Some(i);
        }
        i = i + 1;
    }
    return found;
}

fn show(o: Option(i32)) -> text {
    return match o {
        Option::Some(v) => { "some " + to_text(v) }
        Option::None => { "none" }
    };
}

fn main() {
    let xs: Sequence(i32) = [4, 7, 9];
    print(show(find(xs, 7)) + " " + show(find(xs, 5)));
    let none: Option(i32) = Option::None;
    print(to_text(find(xs, 9) == Option::Some(2)) + " " + to_text(find(xs, 1) == none) + " " + to_text(find(xs, 4) == none));
    return;
}
""")

case("v06_enums", "ok", """
enum Shape {
    Circle(i32),
    Rect(i32, i32),
    Empty
}

fn area(s: Shape) -> i32 {
    return match s {
        Shape::Circle(r) => { 3 * r * r }
        Shape::Rect(w, h) => { w * h }
        Shape::Empty => { 0 }
    };
}

fn main() {
    let shapes: Sequence(Shape) = [Shape::Circle(2), Shape::Rect(3, 4), Shape::Empty];
    let mut total: i32 = 0;
    for s in shapes {
        total = total + area(s);
    }
    print(to_text(total) + " " + to_text(area(Shape::Rect(1, 2))));
    return;
}
""")

QUADS = ["N", "F", "T", "S"]


def quad_table(expr):
    lines = []
    for a in QUADS:
        cells = " + ".join(f'to_text({expr.format(a=a, b=b)})' for b in QUADS)
        lines.append(f'    print("{a}: " + {cells});')
    return "\n".join(lines)


case("v07_quad_lattice", "ok", "fn main() {\n    print(\"and\");\n" + quad_table("{a} && {b}")
     + "\n    print(\"or\");\n" + quad_table("{a} || {b}")
     + "\n    print(\"impl\");\n" + quad_table("{a} -> {b}")
     + "\n    print(to_text(!N) + to_text(!F) + to_text(!T) + to_text(!S));\n    return;\n}\n")

case("v08_quad_truth", "ok", "fn main() {\n    print(\"and\");\n" + quad_table("qtruth_and({a}, {b})")
     + "\n    print(\"or\");\n" + quad_table("qtruth_or({a}, {b})")
     + "\n    print(\"impl\");\n" + quad_table("qtruth_impl({a}, {b})")
     + "\n    print(to_text(qtruth_not(N)) + to_text(qtruth_not(F)) + to_text(qtruth_not(T)) + to_text(qtruth_not(S)));\n    return;\n}\n")

case("v09_quad_match", "ok", """
fn name(q: quad) -> text {
    return match q {
        N => { "unknown" }
        F => { "false" }
        T => { "true" }
        _ => { "conflict" }
    };
}

fn main() {
    let qs: Sequence(quad) = [N, F, T, S];
    for q in qs {
        print(name(q) + " " + to_text(q == T) + " " + to_text(q != N));
    }
    return;
}
""")

case("v10_u32", "ok", """
fn main() {
    let a: u32 = 4000000000u32;
    let b: u32 = 294967295u32;
    print(to_text(a + b) + " " + to_text(a - b) + " " + to_text(65536u32 * 65535u32) + " " + to_text(a / 7u32) + " " + to_text(a % 7u32));
    print(to_text(b < a) + " " + to_text(a <= a) + " " + to_text(a == 4000000000u32));
    return;
}
""")

case("v11_records_in_sequences", "ok", """
record Item {
    name: text,
    qty: i32
}

fn restock(items: Sequence(Item), extra: i32) -> Sequence(Item) {
    let mut out: Sequence(Item) = [];
    for it in items {
        out = push(out, it with { qty: it.qty + extra });
    }
    return out;
}

fn main() {
    let mut items: Sequence(Item) = [Item { name: "a", qty: 1 }, Item { name: "b", qty: 0 }];
    let mut round: i32 = 0;
    while round < 3 {
        items = restock(items, round);
        round = round + 1;
    }
    let mut report: text = "";
    for it in items {
        report = report + it.name + "=" + to_text(it.qty) + " ";
    }
    print(report);
    return;
}
""")

case("v12_text_builder", "ok", """
fn digits(n0: i32) -> text {
    let mut n: i32 = n0;
    let mut out: text = "";
    while n > 0 {
        out = to_text(n % 10) + out;
        n = n / 10;
    }
    return out;
}

fn main() {
    let mut line: text = "";
    let mut i: i32 = 1;
    while i <= 30 {
        line = line + digits(i * 37) + ";";
        i = i + 1;
    }
    print(line);
    print(to_text(text_len(line)));
    return;
}
""")

# ---------------------------------------------------------------- failures
case("x01_assert_fail", "error:assertion failed", """
fn main() {
    print("before");
    assert(2 + 2 == 5);
    print("after");
    return;
}
""")

case("x02_div_zero", "error:runtime trap: DivisionByZero", """
fn main() {
    let z: i32 = 0;
    print(to_text(10 / z));
    return;
}
""")

case("x03_mod_zero", "error:runtime trap: DivisionByZero", """
fn main() {
    let z: i32 = 0;
    print(to_text(10 % z));
    return;
}
""")

case("x04_i32_div_overflow", "error:runtime trap: ArithmeticOverflow", """
fn main() {
    let m: i32 = 0 - 2147483647 - 1;
    let d: i32 = 0 - 1;
    print(to_text(m / d));
    return;
}
""")

case("x05_u32_add_overflow", "error:runtime trap: ArithmeticOverflow", """
fn main() {
    let a: u32 = 4294967295u32;
    print(to_text(a + 1u32));
    return;
}
""")

case("x06_u32_sub_underflow", "error:runtime trap: ArithmeticOverflow", """
fn main() {
    let a: u32 = 3u32;
    print(to_text(a - 4u32));
    return;
}
""")

case("x07_u32_mul_overflow", "error:runtime trap: ArithmeticOverflow", """
fn main() {
    let a: u32 = 65536u32;
    print(to_text(a * a));
    return;
}
""")

case("x08_u32_div_zero", "error:runtime trap: DivisionByZero", """
fn main() {
    let z: u32 = 0u32;
    print(to_text(5u32 / z));
    return;
}
""")

case("x09_steps_quota", "error:quota exceeded: Steps limit=100000 used=100001", """
fn main() {
    let mut i: i32 = 0;
    while true {
        i = i + 1;
    }
    return;
}
""")

case("x10_frames_quota", "error:quota exceeded: Frames limit=256 used=257", """
fn down(n: i32) -> i32 {
    if n == 0 {
        return 0;
    }
    return down(n - 1) + 1;
}

fn main() {
    print(to_text(down(1000)));
    return;
}
""")

case("x11_sequence_index_oob", "error:runtime type mismatch: SEQUENCE_GET index out of bounds: 3", """
fn main() {
    let xs: Sequence(i32) = [1, 2, 3];
    let k: i32 = 3;
    print(to_text(xs[k]));
    return;
}
""")

case("x12_sequence_index_negative", "error:runtime type mismatch: SEQUENCE_GET index must be non-negative", """
fn main() {
    let xs: Sequence(i32) = [1, 2, 3];
    let k: i32 = 0 - 1;
    print(to_text(xs[k]));
    return;
}
""")

case("x13_pop_empty", "error:runtime type mismatch: SEQUENCE_POP source must be non-empty", """
fn main() {
    let xs: Sequence(i32) = [];
    let ys: Sequence(i32) = pop(xs);
    print(to_text(len(ys)));
    return;
}
""")

case("x14_capability_fs_read", "error:ABI FsRead failure [Unavailable]", """
fn main() {
    print(fs_read_text("secret.txt"));
    return;
}
""")

case("x15_capability_stdout_write", "error:ABI StdoutWrite failure [Unavailable]", """
fn main() {
    stdout_write("smuggled");
    return;
}
""")

case("x16_capability_args_read", "error:ABI ArgsRead failure [Unavailable]", """
fn main() {
    print(args_read(0u32));
    return;
}
""")

case("x17_trap_after_output", "error:runtime trap: DivisionByZero", """
fn risky(n: i32) -> i32 {
    print("risky " + to_text(n));
    return 100 / n;
}

fn main() {
    let mut i: i32 = 3;
    while i >= 0 {
        print(to_text(risky(i)));
        i = i - 1;
    }
    return;
}
""")

case("x18_print_marker_rejected", "error:runtime type mismatch: builtin 'print' does not admit host-output marker 'stdout'", """
fn main() {
    print("fine");
    print("stdout");
    return;
}
""")

# ---------------------------------------------------------------- explicitly unsupported
case("u01_map_values", "ok", """
fn main() {
    let mut m: Map(i32, text) = map_empty();
    m = map_set(m, 1, "one");
    print(map_get(m, 1, "none") + " " + map_get(m, 2, "none"));
    return;
}
""", semvm="unsupported")

case("u02_f64", "ok", """
fn main() {
    let x: f64 = 1.5;
    let y: f64 = x * 2.0;
    print("done");
    return;
}
""", semvm="unsupported")

# ---------------------------------------------------------------- scaling workloads
case("w01_counting_loop", "ok", """
fn main() {
    let mut i: i32 = 0;
    let mut acc: i32 = 0;
    while i < 5000 {
        acc = acc + i % 7;
        i = i + 1;
    }
    print(to_text(acc));
    return;
}
""")

case("w02_insertion_sort", "ok", """
fn insert(xs: Sequence(i32), v: i32) -> Sequence(i32) {
    let mut out: Sequence(i32) = [];
    let mut placed: bool = false;
    for x in xs {
        if !placed && v < x {
            out = push(out, v);
            placed = true;
        }
        out = push(out, x);
    }
    if !placed {
        out = push(out, v);
    }
    return out;
}

fn main() {
    let mut xs: Sequence(i32) = [];
    let mut seed: i32 = 17;
    let mut i: i32 = 0;
    while i < 60 {
        seed = (seed * 73 + 41) % 1009;
        xs = insert(xs, seed);
        i = i + 1;
    }
    let mut line: text = "";
    let mut k: i32 = 0;
    while k < 60 {
        line = line + to_text(xs[k]) + " ";
        k = k + 10;
    }
    print(line + to_text(len(xs)));
    return;
}
""")

case("w03_call_heavy", "ok", """
fn add3(a: i32, b: i32, c: i32) -> i32 {
    return a + b + c;
}

fn main() {
    let mut i: i32 = 0;
    let mut s: i32 = 0;
    while i < 2000 {
        s = add3(s, i, 1) % 100000;
        i = i + 1;
    }
    print(to_text(s));
    return;
}
""")

case("w04_quad_batch", "ok", """
fn main() {
    let qs: Sequence(quad) = [N, F, T, S];
    let mut counts: Sequence(i32) = [0, 0, 0, 0];
    let mut round: i32 = 0;
    let mut nn: i32 = 0;
    let mut ff: i32 = 0;
    let mut tt: i32 = 0;
    let mut ss: i32 = 0;
    while round < 50 {
        for a in qs {
            for b in qs {
                let r: quad = (a && b) || (a -> b);
                if r == N { nn = nn + 1; }
                if r == F { ff = ff + 1; }
                if r == T { tt = tt + 1; }
                if r == S { ss = ss + 1; }
            }
        }
        round = round + 1;
    }
    print(to_text(nn) + " " + to_text(ff) + " " + to_text(tt) + " " + to_text(ss));
    return;
}
""")


def main():
    out = HERE / "corpus"
    out.mkdir(exist_ok=True)
    for old in out.glob("*.sm"):
        old.unlink()
    for name, c in CASES.items():
        (out / f"{name}.sm").write_text(c["src"])
    meta = {n: {"expect": c["expect"], "semvm": c["semvm"]} for n, c in CASES.items()}
    (HERE / "corpus.json").write_text(json.dumps(meta, indent=1, sort_keys=True) + "\n")
    print(f"{len(CASES)} cases")


if __name__ == "__main__":
    main()
