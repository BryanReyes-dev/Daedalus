use daedalus_system_info::{
    bounded_text, Architecture, DiscoveryError, DiscoveryStatus, MemoryInfo, OsFamily,
    PlatformInfo, ProcessorInfo, SystemInfoProvider, SystemSnapshot,
};
use sysinfo::{CpuRefreshKind, System};

/// Host operating-system provider backed by `sysinfo`.
///
/// A provider instance retains its `System` object between discovery calls.
/// It enumerates CPU metadata once and refreshes memory on each discovery.
/// It does not enumerate processes, users, disks, networks, or component sensors.
pub struct SysinfoProvider {
    system: Option<System>,
}

impl SysinfoProvider {
    pub const fn new() -> Self {
        Self { system: None }
    }
}

impl Default for SysinfoProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemInfoProvider for SysinfoProvider {
    fn discover(&mut self) -> Result<SystemSnapshot, DiscoveryError> {
        if !sysinfo::IS_SUPPORTED_SYSTEM {
            return Err(DiscoveryError::UnsupportedPlatform);
        }

        if self.system.is_none() {
            let mut system =
                System::new().map_err(|_| DiscoveryError::InitializationFailed)?;
            system.refresh_cpu_list(CpuRefreshKind::everything());
            system.refresh_memory();
            self.system = Some(system);
        }

        let system = self
            .system
            .as_mut()
            .ok_or(DiscoveryError::InitializationFailed)?;

        // Memory availability changes during runtime; CPU identity and topology
        // are retained from the initial enumeration.
        system.refresh_memory();

        let architecture_name = System::cpu_arch();
        let architecture = Architecture::from_name(&architecture_name);

        let platform = PlatformInfo {
            os_family: current_os_family(),
            os_name: System::name().as_deref().map(bounded_text),
            os_version: System::os_version().as_deref().map(bounded_text),
            kernel_version: System::kernel_version().as_deref().map(bounded_text),
            architecture,
            architecture_name: Some(bounded_text(&architecture_name)),
        };

        let cpus = system.cpus();
        let processor = cpus.first().map(|cpu| ProcessorInfo {
            model: non_empty_text(cpu.brand()),
            vendor: non_empty_text(cpu.vendor_id()),
            logical_processors: u32::try_from(cpus.len()).ok(),
            physical_cores: System::physical_core_count()
                .ok()
                .and_then(|count| u32::try_from(count).ok()),
        });

        let memory = MemoryInfo {
            total_bytes: Some(system.total_memory()),
            available_bytes: Some(system.available_memory()),
        };

        let has_baseline_fields = platform.os_name.is_some()
            && platform.architecture != Architecture::Other
            && processor
                .as_ref()
                .and_then(|info| info.logical_processors)
                .is_some()
            && memory.total_bytes.is_some();

        Ok(SystemSnapshot {
            schema_version: 1,
            status: if has_baseline_fields {
                DiscoveryStatus::Complete
            } else {
                DiscoveryStatus::Partial
            },
            platform,
            processor,
            memory: Some(memory),
        })
    }
}

fn non_empty_text(value: &str) -> Option<daedalus_system_info::FixedText<96>> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        None
    } else {
        Some(bounded_text(trimmed))
    }
}

#[cfg(target_os = "windows")]
const fn current_os_family() -> OsFamily {
    OsFamily::Windows
}

#[cfg(target_os = "linux")]
const fn current_os_family() -> OsFamily {
    OsFamily::Linux
}

#[cfg(target_os = "macos")]
const fn current_os_family() -> OsFamily {
    OsFamily::MacOs
}

#[cfg(target_os = "android")]
const fn current_os_family() -> OsFamily {
    OsFamily::Android
}

#[cfg(target_os = "ios")]
const fn current_os_family() -> OsFamily {
    OsFamily::Ios
}

#[cfg(target_os = "freebsd")]
const fn current_os_family() -> OsFamily {
    OsFamily::FreeBsd
}

#[cfg(target_os = "netbsd")]
const fn current_os_family() -> OsFamily {
    OsFamily::NetBsd
}

#[cfg(not(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "macos",
    target_os = "android",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "netbsd"
)))]
const fn current_os_family() -> OsFamily {
    OsFamily::Other
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_host_returns_a_normalized_snapshot() {
        if !sysinfo::IS_SUPPORTED_SYSTEM {
            return;
        }

        let mut provider = SysinfoProvider::new();
        let snapshot = provider
            .discover()
            .expect("supported hosts should provide a system snapshot");

        assert_eq!(snapshot.schema_version, 1);
        assert!(snapshot.processor.is_some());
        assert!(snapshot.memory.is_some());
        assert!(snapshot.platform.architecture_name.is_some());
    }

    #[test]
    fn provider_reuses_its_system_instance() {
        if !sysinfo::IS_SUPPORTED_SYSTEM {
            return;
        }

        let mut provider = SysinfoProvider::new();
        provider.discover().expect("first discovery should succeed");
        provider.discover().expect("second discovery should succeed");
        assert!(provider.system.is_some());
    }
}
