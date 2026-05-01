use std::fmt::Write as _;
use std::time::Instant;

use semantic_core_backend::{
    detect_backend_caps, join_reg32, join_tile128, select_backend, BackendCaps, BackendKind,
};
use semantic_core_capsule::{CoreCapsule, CoreConfig};
use semantic_core_exec::{CoreFunction, CoreProgram, CoreValue, Fx, Instr, RegId};
use semantic_core_quad::{QuadState, QuadTile128, QuadroReg32};
use semantic_core_runtime::{FunctionId, SymbolId};

pub fn run_benchmark(name: &str) -> Result<String, String> {
    match name {
        "quad-reg" => Ok(bench_quad_reg()),
        "tile" => Ok(bench_tile()),
        "exec" => Ok(bench_exec()),
        "all" => Ok(format!(
            "{}\n{}\n{}",
            bench_quad_reg(),
            bench_tile(),
            bench_exec()
        )),
        "caps" => Ok(format_caps_report(BackendKind::Auto, detect_backend_caps())),
        _ => Err(format!("unknown benchmark '{name}'")),
    }
}

pub fn format_caps_report(requested: BackendKind, caps: BackendCaps) -> String {
    let selected = select_backend(requested, caps);
    let arch = std::env::consts::ARCH;
    let yes = |value: bool| if value { "yes" } else { "no" };
    format!(
        "CPU backend report:\n  arch: {arch}\n  popcnt: {}\n  bmi1: {}\n  bmi2: {}\n  avx2: {}\n  avx512: {}\n  neon: {}\n  sve: {}\n  selected backend: {}",
        yes(caps.has_popcnt),
        yes(caps.has_bmi1),
        yes(caps.has_bmi2),
        yes(caps.has_avx2),
        yes(caps.has_avx512),
        yes(caps.has_neon),
        yes(caps.has_sve),
        match selected {
            BackendKind::Scalar => "scalar",
            BackendKind::Auto => "auto",
        }
    )
}

fn bench_quad_reg() -> String {
    let iterations = 100_000u64;
    let regs_per_iter = 64u64;
    let quadits_per_reg = QuadroReg32::LANES as u64;
    let start = Instant::now();
    let mut dst = vec![reg_filled(QuadState::T); regs_per_iter as usize];
    let src = vec![reg_filled(QuadState::F); regs_per_iter as usize];
    for _ in 0..iterations {
        join_reg32(BackendKind::Auto, &mut dst, &src);
    }
    format_bench_line(
        "quad-reg",
        iterations,
        start.elapsed().as_nanos() as u64,
        &[
            ("regs/s", iterations * regs_per_iter),
            ("quadits/s", iterations * regs_per_iter * quadits_per_reg),
        ],
    )
}

fn bench_tile() -> String {
    let iterations = 50_000u64;
    let tiles_per_iter = 32u64;
    let quadits_per_tile = QuadTile128::LANES as u64;
    let start = Instant::now();
    let mut dst = vec![tile_filled(QuadState::T); tiles_per_iter as usize];
    let src = vec![tile_filled(QuadState::F); tiles_per_iter as usize];
    for _ in 0..iterations {
        join_tile128(BackendKind::Auto, &mut dst, &src);
    }
    format_bench_line(
        "tile",
        iterations,
        start.elapsed().as_nanos() as u64,
        &[
            ("tiles/s", iterations * tiles_per_iter),
            ("quadits/s", iterations * tiles_per_iter * quadits_per_tile),
        ],
    )
}

fn bench_exec() -> String {
    let iterations = 10_000u64;
    let start = Instant::now();
    let capsule = CoreCapsule::new(CoreConfig::default());
    let program = sample_program();
    let mut last = CoreValue::Unit;
    let mut last_fuel_used = 0u64;
    for _ in 0..iterations {
        let result = capsule
            .run(&program)
            .expect("sample program should execute");
        last = result.return_value;
        last_fuel_used = result.fuel_used;
    }
    let mut text = format_bench_line(
        "exec",
        iterations,
        start.elapsed().as_nanos() as u64,
        &[
            ("instructions/s", iterations * last_fuel_used),
            ("fuel/s", iterations * last_fuel_used),
        ],
    );
    let _ = write!(text, "\nlast: {:?}", last);
    text
}

fn format_bench_line(name: &str, ops: u64, nanos: u64, extra_rates: &[(&str, u64)]) -> String {
    let nanos = nanos.max(1);
    let ops = ops.max(1);
    let mut text = format!(
        "{name}: ops={ops} ns_total={nanos} ops/s={:.2} ns/op={:.2}",
        rate_per_sec(ops, nanos),
        nanos as f64 / ops as f64,
    );
    for (label, total) in extra_rates {
        let _ = write!(text, " {label}={:.2}", rate_per_sec(*total, nanos));
    }
    text
}

fn rate_per_sec(total: u64, nanos: u64) -> f64 {
    (total as f64 * 1_000_000_000f64) / nanos.max(1) as f64
}

fn reg_filled(state: QuadState) -> QuadroReg32 {
    let mut reg = QuadroReg32::new();
    for lane in 0..QuadroReg32::LANES {
        reg.set_unchecked(lane, state);
    }
    reg
}

fn tile_filled(state: QuadState) -> QuadTile128 {
    let mut tile = QuadTile128::new();
    for lane in 0..QuadTile128::LANES {
        tile.set_unchecked(lane, state);
    }
    tile
}

fn sample_program() -> CoreProgram {
    CoreProgram {
        functions: vec![CoreFunction {
            name_id: SymbolId(0),
            regs: 4,
            instrs: vec![
                Instr::LoadFx {
                    dst: RegId(0),
                    value: Fx::from_raw(2 << 16),
                },
                Instr::LoadFx {
                    dst: RegId(1),
                    value: Fx::from_raw(3 << 16),
                },
                Instr::FxAdd {
                    dst: RegId(2),
                    lhs: RegId(0),
                    rhs: RegId(1),
                },
                Instr::Ret { src: RegId(2) },
            ],
        }],
        entry: FunctionId(0),
    }
}
