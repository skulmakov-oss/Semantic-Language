use semantic_core_quad::{QuadTile128, QuadroReg32};

use crate::CoreBackend;

pub(crate) struct ScalarBackend;

impl CoreBackend for ScalarBackend {
    fn join_reg32(dst: &mut [QuadroReg32], src: &[QuadroReg32]) {
        for (lhs, rhs) in dst.iter_mut().zip(src.iter().copied()) {
            *lhs = lhs.join(rhs);
        }
    }

    fn meet_reg32(dst: &mut [QuadroReg32], src: &[QuadroReg32]) {
        for (lhs, rhs) in dst.iter_mut().zip(src.iter().copied()) {
            *lhs = lhs.meet(rhs);
        }
    }

    fn inverse_reg32(dst: &mut [QuadroReg32]) {
        for reg in dst {
            *reg = reg.inverse();
        }
    }

    fn join_tile128(dst: &mut [QuadTile128], src: &[QuadTile128]) {
        for (lhs, rhs) in dst.iter_mut().zip(src.iter().copied()) {
            *lhs = lhs.join(rhs);
        }
    }

    fn meet_tile128(dst: &mut [QuadTile128], src: &[QuadTile128]) {
        for (lhs, rhs) in dst.iter_mut().zip(src.iter().copied()) {
            *lhs = lhs.meet(rhs);
        }
    }

    fn inverse_tile128(dst: &mut [QuadTile128]) {
        for tile in dst {
            *tile = tile.inverse();
        }
    }
}
