// src/kernel/hal_config.rs (Arquivo principal de configuração)

#[cfg(target_arch = "x86_64")]
#[path = "../arch/x86_64/x86_64_arch.hal"]
pub mod arch_hal;

#[cfg(target_arch = "aarch64")]
#[path = "../arch/arm64/arm64_arch.hal"]
pub mod arch_hal;

// Agora você pode usar:
// use crate::hal_config::arch_hal;
// const KERNEL_START = arch_hal::KERNEL_PHYS_START;
