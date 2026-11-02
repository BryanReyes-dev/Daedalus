//! Portable, allocation-free system information contracts.
//!
//! This crate is intentionally no_std. It defines data types and a provider
//! boundary that host operating systems and embedded board-support packages
//! can implement. It does not inspect hardware itself.

#![no_std]

pub use heapless::{String as FixedText, Vec as FixedVec};

pub const TEXT_CAPACITY: usize = 96;
pub const MAX_STORAGE_DEVICES: usize = 16;
pub const MAX_GPU_DEVICES: usize = 8;
pub const MAX_THERMAL_SENSORS: usize = 32;

pub fn bounded_text(value: &str) -> FixedText<TEXT_CAPACITY> {
    let mut output = FixedText::new();
    for character in value.chars() {
        if output.push(character).is_err() { break; }
    }
    output
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsFamily {
    Windows, Linux, MacOs, Android, Ios, FreeBsd, NetBsd, OpenBsd, DragonFly,
    Solaris, Illumos, Haiku, BareMetal, Other,
}

impl OsFamily {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Windows => "windows", Self::Linux => "linux", Self::MacOs => "macos",
            Self::Android => "android", Self::Ios => "ios", Self::FreeBsd => "freebsd",
            Self::NetBsd => "netbsd", Self::OpenBsd => "openbsd", Self::DragonFly => "dragonfly",
            Self::Solaris => "solaris", Self::Illumos => "illumos", Self::Haiku => "haiku",
            Self::BareMetal => "bare-metal", Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86, X86_64, Arm, Aarch64, RiscV32, RiscV64, PowerPc, PowerPc64,
    S390x, Mips, Mips64, LoongArch64, Wasm32, Wasm64, Other,
}

impl Architecture {
    pub fn from_name(name: &str) -> Self {
        if ["x86", "i386", "i486", "i586", "i686"].iter().any(|v| name.eq_ignore_ascii_case(v)) {
            Self::X86
        } else if name.eq_ignore_ascii_case("x86_64") || name.eq_ignore_ascii_case("amd64") {
            Self::X86_64
        } else if ["arm", "armv6", "armv7", "armv7l"].iter().any(|v| name.eq_ignore_ascii_case(v)) {
            Self::Arm
        } else if name.eq_ignore_ascii_case("aarch64") || name.eq_ignore_ascii_case("arm64") {
            Self::Aarch64
        } else if ["riscv32", "riscv32imac"].iter().any(|v| name.eq_ignore_ascii_case(v)) {
            Self::RiscV32
        } else if ["riscv64", "riscv64gc"].iter().any(|v| name.eq_ignore_ascii_case(v)) {
            Self::RiscV64
        } else if ["powerpc", "ppc"].iter().any(|v| name.eq_ignore_ascii_case(v)) {
            Self::PowerPc
        } else if ["powerpc64", "ppc64", "ppc64le"].iter().any(|v| name.eq_ignore_ascii_case(v)) {
            Self::PowerPc64
        } else if name.eq_ignore_ascii_case("s390x") {
            Self::S390x
        } else if name.eq_ignore_ascii_case("mips") {
            Self::Mips
        } else if name.eq_ignore_ascii_case("mips64") {
            Self::Mips64
        } else if name.eq_ignore_ascii_case("loongarch64") {
            Self::LoongArch64
        } else if name.eq_ignore_ascii_case("wasm32") {
            Self::Wasm32
        } else if name.eq_ignore_ascii_case("wasm64") {
            Self::Wasm64
        } else {
            Self::Other
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::X86 => "x86", Self::X86_64 => "x86_64", Self::Arm => "arm",
            Self::Aarch64 => "aarch64", Self::RiscV32 => "riscv32", Self::RiscV64 => "riscv64",
            Self::PowerPc => "powerpc", Self::PowerPc64 => "powerpc64", Self::S390x => "s390x",
            Self::Mips => "mips", Self::Mips64 => "mips64", Self::LoongArch64 => "loongarch64",
            Self::Wasm32 => "wasm32", Self::Wasm64 => "wasm64", Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryStatus { Complete, Partial }

impl DiscoveryStatus {
    pub const fn as_str(self) -> &'static str {
        match self { Self::Complete => "complete", Self::Partial => "partial" }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformInfo {
    pub os_family: OsFamily,
    pub os_name: Option<FixedText<TEXT_CAPACITY>>,
    pub os_version: Option<FixedText<TEXT_CAPACITY>>,
    pub kernel_version: Option<FixedText<TEXT_CAPACITY>>,
    pub architecture: Architecture,
    pub architecture_name: Option<FixedText<TEXT_CAPACITY>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessorInfo {
    pub model: Option<FixedText<TEXT_CAPACITY>>,
    pub vendor: Option<FixedText<TEXT_CAPACITY>>,
    pub logical_processors: Option<u32>,
    pub physical_cores: Option<u32>,
    pub frequency_mhz: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryInfo {
    pub total_bytes: Option<u64>,
    pub available_bytes: Option<u64>,
    pub used_bytes: Option<u64>,
    pub swap_total_bytes: Option<u64>,
    pub swap_available_bytes: Option<u64>,
    pub swap_used_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductInfo {
    pub vendor: Option<FixedText<TEXT_CAPACITY>>,
    pub name: Option<FixedText<TEXT_CAPACITY>>,
    pub family: Option<FixedText<TEXT_CAPACITY>>,
    pub version: Option<FixedText<TEXT_CAPACITY>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotherboardInfo {
    pub vendor: Option<FixedText<TEXT_CAPACITY>>,
    pub model: Option<FixedText<TEXT_CAPACITY>>,
    pub version: Option<FixedText<TEXT_CAPACITY>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageKind { Hdd, Ssd, Unknown }

impl StorageKind {
    pub const fn as_str(self) -> &'static str {
        match self { Self::Hdd => "hdd", Self::Ssd => "ssd", Self::Unknown => "unknown" }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageDeviceInfo {
    pub name: Option<FixedText<TEXT_CAPACITY>>,
    pub kind: StorageKind,
    pub file_system: Option<FixedText<TEXT_CAPACITY>>,
    pub total_bytes: Option<u64>,
    pub available_bytes: Option<u64>,
    pub removable: bool,
    pub read_only: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpuInfo {
    pub vendor: Option<FixedText<TEXT_CAPACITY>>,
    pub model: Option<FixedText<TEXT_CAPACITY>>,
    pub total_memory_bytes: Option<u64>,
    pub used_memory_bytes: Option<u64>,
    pub usage_percent: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThermalSensorInfo {
    pub label: FixedText<TEXT_CAPACITY>,
    pub temperature_celsius: Option<f32>,
    pub maximum_celsius: Option<f32>,
    pub critical_celsius: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemSnapshot {
    pub schema_version: u16,
    pub status: DiscoveryStatus,
    pub platform: PlatformInfo,
    pub processor: Option<ProcessorInfo>,
    pub memory: Option<MemoryInfo>,
    pub product: Option<ProductInfo>,
    pub motherboard: Option<MotherboardInfo>,
    pub storage_devices: FixedVec<StorageDeviceInfo, MAX_STORAGE_DEVICES>,
    /// None means unsupported; Some(empty) means queried with no devices found.
    pub gpus: Option<FixedVec<GpuInfo, MAX_GPU_DEVICES>>,
    /// None means unsupported; Some(empty) means queried with no sensors found.
    pub thermal_sensors: Option<FixedVec<ThermalSensorInfo, MAX_THERMAL_SENSORS>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryError { UnsupportedPlatform, InitializationFailed, DataUnavailable }

impl DiscoveryError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnsupportedPlatform => "UNSUPPORTED_PLATFORM",
            Self::InitializationFailed => "PROVIDER_INITIALIZATION_FAILED",
            Self::DataUnavailable => "SYSTEM_DATA_UNAVAILABLE",
        }
    }

    pub const fn message(self) -> &'static str {
        match self {
            Self::UnsupportedPlatform => "No system-information provider is available for this target.",
            Self::InitializationFailed => "The system-information provider could not be initialized.",
            Self::DataUnavailable => "The system-information provider could not read system data.",
        }
    }
}

pub trait SystemInfoProvider {
    fn discover(&mut self) -> Result<SystemSnapshot, DiscoveryError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn architecture_names_are_normalized() {
        assert_eq!(Architecture::from_name("AMD64"), Architecture::X86_64);
        assert_eq!(Architecture::from_name("arm64"), Architecture::Aarch64);
        assert_eq!(Architecture::from_name("riscv64"), Architecture::RiscV64);
        assert_eq!(Architecture::from_name("ppc64le"), Architecture::PowerPc64);
        assert_eq!(Architecture::from_name("custom-chip"), Architecture::Other);
    }

    #[test]
    fn bounded_text_truncates_at_utf8_boundaries() {
        let mut input = FixedText::<192>::new();
        for _ in 0..TEXT_CAPACITY { input.push('é').expect("test input fits"); }
        let output = bounded_text(input.as_str());
        assert!(output.len() <= TEXT_CAPACITY);
        assert!(core::str::from_utf8(output.as_bytes()).is_ok());
    }

    #[test]
    fn device_collections_are_bounded() {
        let storage: FixedVec<StorageDeviceInfo, MAX_STORAGE_DEVICES> = FixedVec::new();
        let gpus: FixedVec<GpuInfo, MAX_GPU_DEVICES> = FixedVec::new();
        let sensors: FixedVec<ThermalSensorInfo, MAX_THERMAL_SENSORS> = FixedVec::new();
        assert!(storage.is_empty() && gpus.is_empty() && sensors.is_empty());
        assert_eq!(StorageKind::Ssd.as_str(), "ssd");
    }
}
