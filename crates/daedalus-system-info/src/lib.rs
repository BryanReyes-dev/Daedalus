//! Portable, allocation-free system information contracts.
//!
//! This crate is intentionally `no_std`. It defines the data model and
//! provider boundary that host operating systems and embedded board-support
//! packages can implement. It does not inspect hardware itself.

#![no_std]

pub use heapless::String as FixedText;

/// Maximum UTF-8 byte capacity for a system-information text field.
pub const TEXT_CAPACITY: usize = 96;

/// Create a bounded text value without allocating.
///
/// Values are truncated only at UTF-8 character boundaries. Callers should
/// choose capacities appropriate for the field being populated.
pub fn bounded_text(value: &str) -> FixedText<TEXT_CAPACITY> {
    let mut output = FixedText::new();

    for character in value.chars() {
        if output.push(character).is_err() {
            break;
        }
    }

    output
}

/// Operating-system or execution-environment family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsFamily {
    Windows,
    Linux,
    MacOs,
    Android,
    Ios,
    FreeBsd,
    NetBsd,
    BareMetal,
    Other,
}

impl OsFamily {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Linux => "linux",
            Self::MacOs => "macos",
            Self::Android => "android",
            Self::Ios => "ios",
            Self::FreeBsd => "freebsd",
            Self::NetBsd => "netbsd",
            Self::BareMetal => "bare-metal",
            Self::Other => "other",
        }
    }
}

/// Normalized processor architecture.
///
/// `Other` is paired with `architecture_name` so new architectures do not
/// require a breaking enum change before the runtime can report their name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86,
    X86_64,
    Arm,
    Aarch64,
    RiscV32,
    RiscV64,
    Wasm32,
    Other,
}

impl Architecture {
    pub fn from_name(name: &str) -> Self {
        if name.eq_ignore_ascii_case("x86")
            || name.eq_ignore_ascii_case("i386")
            || name.eq_ignore_ascii_case("i586")
            || name.eq_ignore_ascii_case("i686")
        {
            Self::X86
        } else if name.eq_ignore_ascii_case("x86_64")
            || name.eq_ignore_ascii_case("amd64")
        {
            Self::X86_64
        } else if name.eq_ignore_ascii_case("arm")
            || name.eq_ignore_ascii_case("armv6")
            || name.eq_ignore_ascii_case("armv7")
        {
            Self::Arm
        } else if name.eq_ignore_ascii_case("aarch64")
            || name.eq_ignore_ascii_case("arm64")
        {
            Self::Aarch64
        } else if name.eq_ignore_ascii_case("riscv32") {
            Self::RiscV32
        } else if name.eq_ignore_ascii_case("riscv64") {
            Self::RiscV64
        } else if name.eq_ignore_ascii_case("wasm32") {
            Self::Wasm32
        } else {
            Self::Other
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::X86 => "x86",
            Self::X86_64 => "x86_64",
            Self::Arm => "arm",
            Self::Aarch64 => "aarch64",
            Self::RiscV32 => "riscv32",
            Self::RiscV64 => "riscv64",
            Self::Wasm32 => "wasm32",
            Self::Other => "other",
        }
    }
}

/// Whether the provider returned all baseline fields or only a subset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryStatus {
    Complete,
    Partial,
}

impl DiscoveryStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Partial => "partial",
        }
    }
}

/// Operating-system, execution-environment and architecture information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformInfo {
    pub os_family: OsFamily,
    pub os_name: Option<FixedText<TEXT_CAPACITY>>,
    pub os_version: Option<FixedText<TEXT_CAPACITY>>,
    pub kernel_version: Option<FixedText<TEXT_CAPACITY>>,
    pub architecture: Architecture,
    pub architecture_name: Option<FixedText<TEXT_CAPACITY>>,
}

/// Processor information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessorInfo {
    pub model: Option<FixedText<TEXT_CAPACITY>>,
    pub vendor: Option<FixedText<TEXT_CAPACITY>>,
    pub logical_processors: Option<u32>,
    pub physical_cores: Option<u32>,
}

/// Physical memory information. Values are expressed in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryInfo {
    pub total_bytes: Option<u64>,
    pub available_bytes: Option<u64>,
}

/// A normalized snapshot of the host or embedded target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemSnapshot {
    pub schema_version: u16,
    pub status: DiscoveryStatus,
    pub platform: PlatformInfo,
    pub processor: Option<ProcessorInfo>,
    pub memory: Option<MemoryInfo>,
}

/// Errors that a system-information provider can report without allocating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryError {
    UnsupportedPlatform,
    InitializationFailed,
    DataUnavailable,
}

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
            Self::UnsupportedPlatform => {
                "No system-information provider is available for this target."
            }
            Self::InitializationFailed => {
                "The system-information provider could not be initialized."
            }
            Self::DataUnavailable => {
                "The system-information provider could not read system data."
            }
        }
    }
}

/// Provider boundary implemented by operating-system adapters or embedded
/// board-support packages.
///
/// Implementations may use OS APIs, a BSP, an RTOS interface, or application-
/// supplied hardware readings. The contract has no dependency on `std`,
/// dynamic allocation, Electron, or a specific hardware vendor.
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
        assert_eq!(Architecture::from_name("custom-chip"), Architecture::Other);
    }

    #[test]
    fn bounded_text_truncates_at_utf8_boundaries() {
        let mut long_value = FixedText::<192>::new();
        for _ in 0..TEXT_CAPACITY {
            long_value.push('é').expect("test input fits");
        }
        let bounded = bounded_text(long_value.as_str());

        assert!(bounded.len() <= TEXT_CAPACITY);
        assert!(core::str::from_utf8(bounded.as_bytes()).is_ok());
    }

    #[test]
    fn portable_contract_uses_explicit_availability() {
        let memory = MemoryInfo {
            total_bytes: Some(1024),
            available_bytes: None,
        };

        assert_eq!(memory.total_bytes, Some(1024));
        assert_eq!(memory.available_bytes, None);
        assert_eq!(DiscoveryStatus::Partial.as_str(), "partial");
    }
}
