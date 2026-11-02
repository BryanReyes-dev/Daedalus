use daedalus_system_info::{
    bounded_text, Architecture, DiscoveryError, DiscoveryStatus, FixedText, FixedVec, GpuInfo,
    MemoryInfo, MotherboardInfo, OsFamily, PlatformInfo, ProcessorInfo, ProductInfo,
    StorageDeviceInfo, StorageKind, SystemInfoProvider, SystemSnapshot, ThermalSensorInfo,
    MAX_GPU_DEVICES, MAX_STORAGE_DEVICES, MAX_THERMAL_SENSORS, TEXT_CAPACITY,
};
use sysinfo::{
    Components, CpuRefreshKind, DiskKind, Disks, Error as SysinfoError, Gpus,
    MemoryRefreshKind, Motherboard, Product, System,
};

/// Host discovery adapter. It does not request process, user, network, host
/// name, mount path, serial number, UUID, asset tag, or PCI address data.
pub struct SysinfoProvider {
    system: Option<System>,
    disks: Disks,
    components: Option<Components>,
    gpus: Option<Gpus>,
}

impl SysinfoProvider {
    pub fn new() -> Self {
        Self {
            system: None,
            disks: Disks::new(),
            components: Components::new().ok(),
            // sysinfo currently provides GPU inventory on Windows, Linux and macOS.
            gpus: Gpus::new().ok(),
        }
    }
}

impl Default for SysinfoProvider {
    fn default() -> Self { Self::new() }
}

impl SystemInfoProvider for SysinfoProvider {
    fn discover(&mut self) -> Result<SystemSnapshot, DiscoveryError> {
        if self.system.is_none() {
            let mut system = System::new().map_err(map_sysinfo_error)?;
            system.refresh_cpu_list(CpuRefreshKind::nothing().with_frequency());
            self.system = Some(system);
        }

        let system = self.system.as_mut().ok_or(DiscoveryError::InitializationFailed)?;
        system.refresh_memory_specifics(
            MemoryRefreshKind::nothing().with_ram().with_swap()
        );
        system.refresh_cpu_specifics(CpuRefreshKind::nothing().with_frequency());

        let architecture_name = System::cpu_arch();
        let platform = PlatformInfo {
            os_family: current_os_family(),
            os_name: optional_text(System::name()),
            os_version: optional_text(System::os_version()),
            kernel_version: optional_text(System::kernel_version()),
            architecture: Architecture::from_name(&architecture_name),
            architecture_name: non_empty_text(&architecture_name),
        };

        let cpus = system.cpus();
        let processor = cpus.first().map(|cpu| ProcessorInfo {
            model: non_empty_text(cpu.brand()),
            vendor: non_empty_text(cpu.vendor_id()),
            logical_processors: u32::try_from(cpus.len()).ok(),
            physical_cores: System::physical_core_count()
                .ok()
                .and_then(|count| u32::try_from(count).ok()),
            frequency_mhz: non_zero(cpu.frequency()),
        });

        let memory = MemoryInfo {
            total_bytes: non_zero(system.total_memory()),
            available_bytes: Some(system.available_memory()),
            used_bytes: Some(system.used_memory()),
            swap_total_bytes: Some(system.total_swap()),
            swap_available_bytes: Some(system.free_swap()),
            swap_used_bytes: Some(system.used_swap()),
        };

        let product = product_info();
        let motherboard = motherboard_info();

        // Refresh inventory and remove devices that are no longer present.
        self.disks.refresh(true);
        let mut storage_devices: FixedVec<StorageDeviceInfo, MAX_STORAGE_DEVICES> =
            FixedVec::new();
        for disk in &self.disks {
            let kind = match disk.kind() {
                DiskKind::HDD => StorageKind::Hdd,
                DiskKind::SSD => StorageKind::Ssd,
                DiskKind::Unknown(_) => StorageKind::Unknown,
            };
            let device = StorageDeviceInfo {
                name: non_empty_text(&disk.name().to_string_lossy()),
                kind,
                file_system: non_empty_text(&disk.file_system().to_string_lossy()),
                total_bytes: non_zero(disk.total_space()),
                available_bytes: Some(disk.available_space()),
                removable: disk.is_removable(),
                read_only: disk.is_read_only(),
            };
            if storage_devices.push(device).is_err() { break; }
        }

        if let Some(gpus) = self.gpus.as_mut() {
            gpus.refresh(true);
        }
        let gpus = self.gpus.as_ref().map(|inventory| {
            let mut devices: FixedVec<GpuInfo, MAX_GPU_DEVICES> = FixedVec::new();
            for gpu in inventory {
                let device = GpuInfo {
                    vendor: gpu.vendor().and_then(non_empty_text),
                    model: gpu.model().and_then(non_empty_text),
                    total_memory_bytes: gpu.total_memory(),
                    used_memory_bytes: gpu.used_memory(),
                    usage_percent: finite_percentage(gpu.usage()),
                };
                if devices.push(device).is_err() { break; }
            }
            devices
        });

        if let Some(components) = self.components.as_mut() {
            components.refresh(true);
        }
        let thermal_sensors = self.components.as_ref().map(|components| {
            let mut sensors: FixedVec<ThermalSensorInfo, MAX_THERMAL_SENSORS> =
                FixedVec::new();
            for component in components {
                let sensor = ThermalSensorInfo {
                    label: bounded_text(component.label()),
                    temperature_celsius: finite_value(component.temperature()),
                    maximum_celsius: finite_value(component.max()),
                    critical_celsius: finite_value(component.critical()),
                };
                if sensors.push(sensor).is_err() { break; }
            }
            sensors
        });

        let baseline_complete = platform.os_name.is_some()
            && platform.architecture_name.is_some()
            && processor.as_ref().and_then(|cpu| cpu.logical_processors).is_some()
            && memory.total_bytes.is_some();

        Ok(SystemSnapshot {
            schema_version: 1,
            status: if baseline_complete { DiscoveryStatus::Complete } else { DiscoveryStatus::Partial },
            platform,
            processor,
            memory: Some(memory),
            product,
            motherboard,
            storage_devices,
            gpus,
            thermal_sensors,
        })
    }
}

fn product_info() -> Option<ProductInfo> {
    let product = ProductInfo {
        vendor: optional_text(Product::vendor_name()),
        name: optional_text(Product::name()),
        family: optional_text(Product::family()),
        version: optional_text(Product::version()),
    };
    if product.vendor.is_some() || product.name.is_some()
        || product.family.is_some() || product.version.is_some() {
        Some(product)
    } else {
        None
    }
}

fn motherboard_info() -> Option<MotherboardInfo> {
    let board = Motherboard::new().ok()?;
    let info = MotherboardInfo {
        vendor: board.vendor_name().as_deref().and_then(non_empty_text),
        model: board.name().as_deref().and_then(non_empty_text),
        version: board.version().as_deref().and_then(non_empty_text),
    };
    if info.vendor.is_some() || info.model.is_some() || info.version.is_some() {
        Some(info)
    } else {
        None
    }
}

fn map_sysinfo_error(error: SysinfoError) -> DiscoveryError {
    match error {
        SysinfoError::Unsupported => DiscoveryError::UnsupportedPlatform,
        SysinfoError::Io(_) | SysinfoError::Other(_) => DiscoveryError::InitializationFailed,
    }
}

fn optional_text(value: Result<String, SysinfoError>) -> Option<FixedText<TEXT_CAPACITY>> {
    value.ok().and_then(|value| non_empty_text(&value))
}

fn non_zero(value: u64) -> Option<u64> {
    if value == 0 { None } else { Some(value) }
}

fn non_empty_text(value: &str) -> Option<FixedText<TEXT_CAPACITY>> {
    let value = value.trim();
    if value.is_empty() { None } else { Some(bounded_text(value)) }
}

fn finite_value(value: Option<f32>) -> Option<f32> {
    value.filter(|number| number.is_finite())
}

fn finite_percentage(value: Option<f32>) -> Option<f32> {
    finite_value(value).map(|number| number.clamp(0.0, 100.0))
}

#[cfg(target_os = "windows")]
const fn current_os_family() -> OsFamily { OsFamily::Windows }
#[cfg(target_os = "linux")]
const fn current_os_family() -> OsFamily { OsFamily::Linux }
#[cfg(target_os = "macos")]
const fn current_os_family() -> OsFamily { OsFamily::MacOs }
#[cfg(target_os = "android")]
const fn current_os_family() -> OsFamily { OsFamily::Android }
#[cfg(target_os = "ios")]
const fn current_os_family() -> OsFamily { OsFamily::Ios }
#[cfg(target_os = "freebsd")]
const fn current_os_family() -> OsFamily { OsFamily::FreeBsd }
#[cfg(target_os = "netbsd")]
const fn current_os_family() -> OsFamily { OsFamily::NetBsd }
#[cfg(target_os = "openbsd")]
const fn current_os_family() -> OsFamily { OsFamily::OpenBsd }
#[cfg(target_os = "dragonfly")]
const fn current_os_family() -> OsFamily { OsFamily::DragonFly }
#[cfg(target_os = "solaris")]
const fn current_os_family() -> OsFamily { OsFamily::Solaris }
#[cfg(target_os = "illumos")]
const fn current_os_family() -> OsFamily { OsFamily::Illumos }
#[cfg(target_os = "haiku")]
const fn current_os_family() -> OsFamily { OsFamily::Haiku }
#[cfg(not(any(
    target_os = "windows", target_os = "linux", target_os = "macos",
    target_os = "android", target_os = "ios", target_os = "freebsd",
    target_os = "netbsd", target_os = "openbsd", target_os = "dragonfly",
    target_os = "solaris", target_os = "illumos", target_os = "haiku"
)))]
const fn current_os_family() -> OsFamily { OsFamily::Other }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_sensor_values_are_filtered_and_usage_is_clamped() {
        assert_eq!(finite_value(Some(f32::NAN)), None);
        assert_eq!(finite_value(Some(42.0)), Some(42.0));
        assert_eq!(finite_percentage(Some(130.0)), Some(100.0));
    }

    #[test]
    fn zero_values_are_not_reported_as_detected_capacities() {
        assert_eq!(non_zero(0), None);
        assert_eq!(non_zero(1024), Some(1024));
    }

    #[test]
    fn optional_text_filters_empty_values() {
        assert!(non_empty_text("  ").is_none());
        assert_eq!(non_empty_text("  model ").unwrap().as_str(), "model");
    }
}
