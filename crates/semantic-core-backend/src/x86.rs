use crate::BackendCaps;

pub fn detect_x86_caps() -> BackendCaps {
    #[cfg(all(feature = "std", any(target_arch = "x86", target_arch = "x86_64")))]
    {
        BackendCaps {
            has_popcnt: std::arch::is_x86_feature_detected!("popcnt"),
            has_bmi1: std::arch::is_x86_feature_detected!("bmi1"),
            has_bmi2: std::arch::is_x86_feature_detected!("bmi2"),
            has_avx2: std::arch::is_x86_feature_detected!("avx2"),
            has_avx512: std::arch::is_x86_feature_detected!("avx512f"),
            has_neon: false,
            has_sve: false,
        }
    }
    #[cfg(not(all(feature = "std", any(target_arch = "x86", target_arch = "x86_64"))))]
    {
        BackendCaps::scalar()
    }
}
