use super::{IrModule, OptPass, OptReport};
use crate::frontend::QuadVal;
use crate::legacy_lowering::IrInstr;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CrystalFoldPass;

impl OptPass for CrystalFoldPass {
    fn name(&self) -> &'static str {
        "CrystalFold"
    }

    fn version(&self) -> u32 {
        1
    }

    fn run(&self, ir: &mut IrModule) -> OptReport {
        let mut rewrites = 0u32;
        for func in &mut ir.functions {
            rewrites = rewrites.saturating_add(fold_constants_and_identities(&mut func.instrs));
        }
        OptReport {
            changed: rewrites > 0,
            num_rewrites: rewrites,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ConstVal {
    Quad(QuadVal),
    Bool(bool),
    F64(f64),
    I32(i32),
    U32(u32),
    Fx(i32),
}

fn fold_constants_and_identities(instrs: &mut Vec<IrInstr>) -> u32 {
    let mut rewrites = 0u32;
    let mut out = Vec::with_capacity(instrs.len());
    let mut cst: HashMap<u16, ConstVal> = HashMap::new();

    for instr in instrs.drain(..) {
        match instr {
            IrInstr::Label { name } => {
                cst.clear();
                out.push(IrInstr::Label { name });
            }
            IrInstr::Jmp { label } => {
                cst.clear();
                out.push(IrInstr::Jmp { label });
            }
            IrInstr::JmpIf { cond, label } => {
                cst.clear();
                out.push(IrInstr::JmpIf { cond, label });
            }
            IrInstr::Assert { cond } => {
                cst.clear();
                out.push(IrInstr::Assert { cond });
            }
            IrInstr::Call { dst, name, args } => {
                cst.clear();
                out.push(IrInstr::Call { dst, name, args });
            }
            IrInstr::GateRead {
                dst,
                device_id,
                port,
            } => {
                cst.remove(&dst);
                out.push(IrInstr::GateRead {
                    dst,
                    device_id,
                    port,
                });
            }
            IrInstr::GateWrite {
                device_id,
                port,
                src,
            } => {
                out.push(IrInstr::GateWrite {
                    device_id,
                    port,
                    src,
                });
            }
            IrInstr::PulseEmit { signal } => {
                out.push(IrInstr::PulseEmit { signal });
            }
            IrInstr::Ret { src } => {
                cst.clear();
                out.push(IrInstr::Ret { src });
            }
            IrInstr::LoadQ { dst, val } => {
                cst.insert(dst, ConstVal::Quad(val));
                out.push(IrInstr::LoadQ { dst, val });
            }
            IrInstr::LoadBool { dst, val } => {
                cst.insert(dst, ConstVal::Bool(val));
                out.push(IrInstr::LoadBool { dst, val });
            }
            IrInstr::LoadI32 { dst, val } => {
                cst.insert(dst, ConstVal::I32(val));
                out.push(IrInstr::LoadI32 { dst, val });
            }
            IrInstr::LoadU32 { dst, val } => {
                cst.insert(dst, ConstVal::U32(val));
                out.push(IrInstr::LoadU32 { dst, val });
            }
            IrInstr::LoadF64 { dst, val } => {
                cst.insert(dst, ConstVal::F64(val));
                out.push(IrInstr::LoadF64 { dst, val });
            }
            IrInstr::LoadFx { dst, val } => {
                cst.insert(dst, ConstVal::Fx(val));
                out.push(IrInstr::LoadFx { dst, val });
            }
            IrInstr::MakeTuple { dst, items } => {
                cst.remove(&dst);
                out.push(IrInstr::MakeTuple { dst, items });
            }
            IrInstr::TupleGet { dst, src, index } => {
                cst.remove(&dst);
                out.push(IrInstr::TupleGet { dst, src, index });
            }
            IrInstr::LoadVar { dst, name } => {
                cst.remove(&dst);
                out.push(IrInstr::LoadVar { dst, name });
            }
            IrInstr::StoreVar { name, src } => {
                out.push(IrInstr::StoreVar { name, src });
            }
            IrInstr::BoolNot { dst, src } => {
                if let Some(ConstVal::Bool(b)) = cst.get(&src).copied() {
                    rewrites = rewrites.saturating_add(1);
                    cst.insert(dst, ConstVal::Bool(!b));
                    out.push(IrInstr::LoadBool { dst, val: !b });
                } else {
                    cst.remove(&dst);
                    out.push(IrInstr::BoolNot { dst, src });
                }
            }
            IrInstr::QNot { dst, src } => {
                if let Some(ConstVal::Quad(q)) = cst.get(&src).copied() {
                    rewrites = rewrites.saturating_add(1);
                    let nq = quad_not_const(q);
                    cst.insert(dst, ConstVal::Quad(nq));
                    out.push(IrInstr::LoadQ { dst, val: nq });
                } else {
                    cst.remove(&dst);
                    out.push(IrInstr::QNot { dst, src });
                }
            }
            IrInstr::BoolAnd { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::Bool(a)), Some(ConstVal::Bool(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::Bool(a && b));
                        out.push(IrInstr::LoadBool { dst, val: a && b });
                    }
                    _ if dst == lhs && matches!(cst.get(&rhs), Some(ConstVal::Bool(true))) => {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if dst == rhs && matches!(cst.get(&lhs), Some(ConstVal::Bool(true))) => {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if matches!(cst.get(&lhs), Some(ConstVal::Bool(false)))
                        || matches!(cst.get(&rhs), Some(ConstVal::Bool(false))) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::Bool(false));
                        out.push(IrInstr::LoadBool { dst, val: false });
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::BoolAnd { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::BoolOr { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::Bool(a)), Some(ConstVal::Bool(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::Bool(a || b));
                        out.push(IrInstr::LoadBool { dst, val: a || b });
                    }
                    _ if dst == lhs && matches!(cst.get(&rhs), Some(ConstVal::Bool(false))) => {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if dst == rhs && matches!(cst.get(&lhs), Some(ConstVal::Bool(false))) => {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if matches!(cst.get(&lhs), Some(ConstVal::Bool(true)))
                        || matches!(cst.get(&rhs), Some(ConstVal::Bool(true))) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::Bool(true));
                        out.push(IrInstr::LoadBool { dst, val: true });
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::BoolOr { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::QAnd { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::Quad(a)), Some(ConstVal::Quad(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        let v = quad_and_const(a, b);
                        cst.insert(dst, ConstVal::Quad(v));
                        out.push(IrInstr::LoadQ { dst, val: v });
                    }
                    _ if dst == lhs && matches!(cst.get(&rhs), Some(ConstVal::Quad(QuadVal::S))) => {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if dst == rhs && matches!(cst.get(&lhs), Some(ConstVal::Quad(QuadVal::S))) => {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if matches!(cst.get(&lhs), Some(ConstVal::Quad(QuadVal::N)))
                        || matches!(cst.get(&rhs), Some(ConstVal::Quad(QuadVal::N))) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::Quad(QuadVal::N));
                        out.push(IrInstr::LoadQ {
                            dst,
                            val: QuadVal::N,
                        });
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::QAnd { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::QOr { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::Quad(a)), Some(ConstVal::Quad(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        let v = quad_or_const(a, b);
                        cst.insert(dst, ConstVal::Quad(v));
                        out.push(IrInstr::LoadQ { dst, val: v });
                    }
                    _ if dst == lhs && matches!(cst.get(&rhs), Some(ConstVal::Quad(QuadVal::N))) => {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if dst == rhs && matches!(cst.get(&lhs), Some(ConstVal::Quad(QuadVal::N))) => {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if matches!(cst.get(&lhs), Some(ConstVal::Quad(QuadVal::S)))
                        || matches!(cst.get(&rhs), Some(ConstVal::Quad(QuadVal::S))) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::Quad(QuadVal::S));
                        out.push(IrInstr::LoadQ {
                            dst,
                            val: QuadVal::S,
                        });
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::QOr { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::QImpl { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::Quad(a)), Some(ConstVal::Quad(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        let v = quad_or_const(quad_not_const(a), b);
                        cst.insert(dst, ConstVal::Quad(v));
                        out.push(IrInstr::LoadQ { dst, val: v });
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::QImpl { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::CmpEq { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(a), Some(b)) => {
                        rewrites = rewrites.saturating_add(1);
                        let eq = const_eq(a, b);
                        cst.insert(dst, ConstVal::Bool(eq));
                        out.push(IrInstr::LoadBool { dst, val: eq });
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::CmpEq { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::CmpNe { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(a), Some(b)) => {
                        rewrites = rewrites.saturating_add(1);
                        let ne = !const_eq(a, b);
                        cst.insert(dst, ConstVal::Bool(ne));
                        out.push(IrInstr::LoadBool { dst, val: ne });
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::CmpNe { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::AddF64 { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::F64(a)), Some(ConstVal::F64(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::F64(a + b));
                        out.push(IrInstr::LoadF64 { dst, val: a + b });
                    }
                    _ if dst == lhs
                        && matches!(cst.get(&rhs), Some(ConstVal::F64(v)) if *v == 0.0) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if dst == rhs
                        && matches!(cst.get(&lhs), Some(ConstVal::F64(v)) if *v == 0.0) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::AddF64 { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::SubF64 { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::F64(a)), Some(ConstVal::F64(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::F64(a - b));
                        out.push(IrInstr::LoadF64 { dst, val: a - b });
                    }
                    _ if dst == lhs
                        && matches!(cst.get(&rhs), Some(ConstVal::F64(v)) if *v == 0.0) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::SubF64 { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::MulF64 { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::F64(a)), Some(ConstVal::F64(b))) => {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::F64(a * b));
                        out.push(IrInstr::LoadF64 { dst, val: a * b });
                    }
                    _ if dst == lhs
                        && matches!(cst.get(&rhs), Some(ConstVal::F64(v)) if *v == 1.0) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ if dst == rhs
                        && matches!(cst.get(&lhs), Some(ConstVal::F64(v)) if *v == 1.0) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::MulF64 { dst, lhs, rhs });
                    }
                }
            }
            IrInstr::DivF64 { dst, lhs, rhs } => {
                match (cst.get(&lhs).copied(), cst.get(&rhs).copied()) {
                    (Some(ConstVal::F64(a)), Some(ConstVal::F64(b))) if b != 0.0 => {
                        rewrites = rewrites.saturating_add(1);
                        cst.insert(dst, ConstVal::F64(a / b));
                        out.push(IrInstr::LoadF64 { dst, val: a / b });
                    }
                    _ if dst == lhs
                        && matches!(cst.get(&rhs), Some(ConstVal::F64(v)) if *v == 1.0) =>
                    {
                        rewrites = rewrites.saturating_add(1);
                    }
                    _ => {
                        cst.remove(&dst);
                        out.push(IrInstr::DivF64 { dst, lhs, rhs });
                    }
                }
            }
        }
    }
    *instrs = out;
    rewrites
}

fn quad_to_u8_const(q: QuadVal) -> u8 {
    match q {
        QuadVal::N => 0,
        QuadVal::F => 1,
        QuadVal::T => 2,
        QuadVal::S => 3,
    }
}

fn u8_to_quad_const(v: u8) -> QuadVal {
    match v & 0b11 {
        0 => QuadVal::N,
        1 => QuadVal::F,
        2 => QuadVal::T,
        _ => QuadVal::S,
    }
}

fn quad_not_const(a: QuadVal) -> QuadVal {
    let v = quad_to_u8_const(a);
    let r = ((v & 0b10) >> 1) | ((v & 0b01) << 1);
    u8_to_quad_const(r)
}

fn quad_and_const(a: QuadVal, b: QuadVal) -> QuadVal {
    u8_to_quad_const(quad_to_u8_const(a) & quad_to_u8_const(b))
}

fn quad_or_const(a: QuadVal, b: QuadVal) -> QuadVal {
    u8_to_quad_const(quad_to_u8_const(a) | quad_to_u8_const(b))
}

fn const_eq(a: ConstVal, b: ConstVal) -> bool {
    match (a, b) {
        (ConstVal::Quad(x), ConstVal::Quad(y)) => x == y,
        (ConstVal::Bool(x), ConstVal::Bool(y)) => x == y,
        (ConstVal::F64(x), ConstVal::F64(y)) => x == y,
        (ConstVal::I32(x), ConstVal::I32(y)) => x == y,
        (ConstVal::Fx(x), ConstVal::Fx(y)) => x == y,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy_lowering::{IrFunction, IrInstr};

    #[test]
    fn crystalfold_idempotent() {
        let pass = CrystalFoldPass;
        let base = IrFunction {
            name: "main".to_string(),
            instrs: vec![
                IrInstr::LoadBool { dst: 0, val: true },
                IrInstr::LoadBool { dst: 1, val: false },
                IrInstr::BoolAnd {
                    dst: 2,
                    lhs: 0,
                    rhs: 1,
                },
                IrInstr::LoadF64 { dst: 3, val: 2.0 },
                IrInstr::LoadF64 { dst: 4, val: 3.0 },
                IrInstr::AddF64 {
                    dst: 5,
                    lhs: 3,
                    rhs: 4,
                },
                IrInstr::Ret { src: Some(5) },
            ],
        };

        let mut m1 = IrModule {
            functions: vec![base.clone()],
        };
        let r1 = pass.run(&mut m1);
        assert!(r1.changed);

        let mut m2 = m1.clone();
        let r2 = pass.run(&mut m2);
        assert!(!r2.changed);
        assert_eq!(m1, m2);
    }
}

