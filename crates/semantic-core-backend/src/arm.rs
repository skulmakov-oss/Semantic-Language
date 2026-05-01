use crate::BackendCaps;

pub fn detect_arm_caps() -> BackendCaps {
    #[cfg(all(feature = "std", target_arch = "aarch64"))]
    {
        BackendCaps {
            has_popcnt: false,
            has_bmi1: false,
            has_bmi2: false,
            has_avx2: false,
            has_avx512: false,
            has_neon: std::arch::is_aarch64_feature_detected!("neon"),
            has_sve: std::arch::is_aarch64_feature_detected!("sve"),
        }
    }
    #[cfg(not(all(feature = "std", target_arch = "aarch64")))]
    {
        BackendCaps::scalar()
    }
}
