//! Host-side runtime command dispatcher.

use daedalus_runtime_protocol::{RuntimeRequest, RuntimeResponse, CURRENT_PROTOCOL_VERSION};
use daedalus_system_discovery::SysinfoProvider;
use daedalus_system_info::{
    FixedText, SystemInfoProvider, SystemSnapshot, TEXT_CAPACITY,
};
use serde_json::{json, Value};

pub struct RuntimeEngine {
    system_info_provider: Box<dyn SystemInfoProvider>,
}

impl RuntimeEngine {
    pub fn new() -> Self {
        Self::with_system_info_provider(SysinfoProvider::new())
    }

    /// Inject a host, test, RTOS, or board-support provider.
    pub fn with_system_info_provider<P>(provider: P) -> Self
    where
        P: SystemInfoProvider + 'static,
    {
        Self { system_info_provider: Box::new(provider) }
    }

    pub fn dispatch(&mut self, request: RuntimeRequest) -> RuntimeResponse {
        let request_id = request.request_id;
        if request.protocol_version != CURRENT_PROTOCOL_VERSION {
            return RuntimeResponse::failure(
                request_id,
                "PROTOCOL_VERSION_UNSUPPORTED",
                format!(
                    "Client protocol version {} is unsupported; runtime protocol version is {}.",
                    request.protocol_version, CURRENT_PROTOCOL_VERSION
                ),
            );
        }

        match request.method.as_str() {
            "runtime.ping" => RuntimeResponse::success(
                request_id,
                json!({"alive": true, "protocolVersion": CURRENT_PROTOCOL_VERSION}),
            ),
            "runtime.getInfo" => RuntimeResponse::success(
                request_id,
                json!({
                    "name": "Daedalus Runtime",
                    "version": env!("CARGO_PKG_VERSION"),
                    "status": "ready",
                    "protocolVersion": CURRENT_PROTOCOL_VERSION,
                    "capabilities": ["runtime.ping", "runtime.getInfo", "hardware.discover"]
                }),
            ),
            "hardware.discover" => match self.system_info_provider.discover() {
                Ok(snapshot) => RuntimeResponse::success(request_id, snapshot_to_json(&snapshot)),
                Err(error) => RuntimeResponse::failure(request_id, error.code(), error.message()),
            },
            method => RuntimeResponse::failure(
                request_id,
                "METHOD_NOT_FOUND",
                format!("Runtime method '{method}' is not available."),
            ),
        }
    }
}

impl Default for RuntimeEngine {
    fn default() -> Self { Self::new() }
}

fn text(value: &Option<FixedText<TEXT_CAPACITY>>) -> Option<&str> {
    value.as_ref().map(|item| item.as_str())
}

fn snapshot_to_json(snapshot: &SystemSnapshot) -> Value {
    let processor = snapshot.processor.as_ref().map(|cpu| json!({
        "model": text(&cpu.model),
        "vendor": text(&cpu.vendor),
        "logicalProcessors": cpu.logical_processors,
        "physicalCores": cpu.physical_cores,
        "frequencyMhz": cpu.frequency_mhz
    }));

    let memory = snapshot.memory.map(|memory| json!({
        "totalBytes": memory.total_bytes,
        "availableBytes": memory.available_bytes,
        "usedBytes": memory.used_bytes,
        "swapTotalBytes": memory.swap_total_bytes,
        "swapAvailableBytes": memory.swap_available_bytes,
        "swapUsedBytes": memory.swap_used_bytes
    }));

    let product = snapshot.product.as_ref().map(|product| json!({
        "vendor": text(&product.vendor),
        "name": text(&product.name),
        "family": text(&product.family),
        "version": text(&product.version)
    }));

    let motherboard = snapshot.motherboard.as_ref().map(|board| json!({
        "vendor": text(&board.vendor),
        "model": text(&board.model),
        "version": text(&board.version)
    }));

    let storage_devices = snapshot.storage_devices.iter().map(|disk| json!({
        "name": text(&disk.name),
        "kind": disk.kind.as_str(),
        "fileSystem": text(&disk.file_system),
        "totalBytes": disk.total_bytes,
        "availableBytes": disk.available_bytes,
        "removable": disk.removable,
        "readOnly": disk.read_only
    })).collect::<Vec<_>>();

    let gpus = snapshot.gpus.as_ref().map(|devices| {
        devices.iter().map(|gpu| json!({
            "vendor": text(&gpu.vendor),
            "model": text(&gpu.model),
            "totalMemoryBytes": gpu.total_memory_bytes,
            "usedMemoryBytes": gpu.used_memory_bytes,
            "usagePercent": gpu.usage_percent
        })).collect::<Vec<_>>()
    });

    let sensors = snapshot.thermal_sensors.as_ref().map(|items| {
        items.iter().map(|sensor| json!({
            "label": sensor.label.as_str(),
            "temperatureCelsius": sensor.temperature_celsius,
            "maximumCelsius": sensor.maximum_celsius,
            "criticalCelsius": sensor.critical_celsius
        })).collect::<Vec<_>>()
    });

    json!({
        "schemaVersion": snapshot.schema_version,
        "status": snapshot.status.as_str(),
        "platform": {
            "osFamily": snapshot.platform.os_family.as_str(),
            "osName": text(&snapshot.platform.os_name),
            "osVersion": text(&snapshot.platform.os_version),
            "kernelVersion": text(&snapshot.platform.kernel_version),
            "architecture": snapshot.platform.architecture.as_str(),
            "architectureName": text(&snapshot.platform.architecture_name)
        },
        "processor": processor,
        "memory": memory,
        "product": product,
        "motherboard": motherboard,
        "storageDevices": storage_devices,
        "gpus": gpus,
        "thermalSensors": sensors
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use daedalus_system_info::{
        Architecture, DiscoveryError, DiscoveryStatus, FixedVec, GpuInfo, MemoryInfo, OsFamily,
        PlatformInfo, ProcessorInfo, StorageDeviceInfo, ThermalSensorInfo, MAX_GPU_DEVICES,
        MAX_STORAGE_DEVICES, MAX_THERMAL_SENSORS,
    };

    struct MockProvider(Result<SystemSnapshot, DiscoveryError>);

    impl SystemInfoProvider for MockProvider {
        fn discover(&mut self) -> Result<SystemSnapshot, DiscoveryError> {
            self.0.clone()
        }
    }

    fn request(version: u16, id: &str, method: &str) -> RuntimeRequest {
        RuntimeRequest {
            protocol_version: version,
            request_id: id.to_owned(),
            method: method.to_owned(),
            params: json!({}),
        }
    }

    fn sample() -> SystemSnapshot {
        let mut os = FixedText::<96>::new();
        os.push_str("Test OS").unwrap();
        let mut arch = FixedText::<96>::new();
        arch.push_str("x86_64").unwrap();
        let mut cpu = FixedText::<96>::new();
        cpu.push_str("Test CPU").unwrap();
        let storage: FixedVec<StorageDeviceInfo, MAX_STORAGE_DEVICES> = FixedVec::new();
        let gpus: Option<FixedVec<GpuInfo, MAX_GPU_DEVICES>> = Some(FixedVec::new());
        let sensors: Option<FixedVec<ThermalSensorInfo, MAX_THERMAL_SENSORS>> =
            Some(FixedVec::new());

        SystemSnapshot {
            schema_version: 1,
            status: DiscoveryStatus::Complete,
            platform: PlatformInfo {
                os_family: OsFamily::Linux,
                os_name: Some(os),
                os_version: None,
                kernel_version: None,
                architecture: Architecture::X86_64,
                architecture_name: Some(arch),
            },
            processor: Some(ProcessorInfo {
                model: Some(cpu),
                vendor: None,
                logical_processors: Some(8),
                physical_cores: Some(4),
                frequency_mhz: Some(3600),
            }),
            memory: Some(MemoryInfo {
                total_bytes: Some(16 * 1024 * 1024 * 1024),
                available_bytes: Some(8 * 1024 * 1024 * 1024),
                used_bytes: Some(8 * 1024 * 1024 * 1024),
                swap_total_bytes: Some(0),
                swap_available_bytes: Some(0),
                swap_used_bytes: Some(0),
            }),
            product: None,
            motherboard: None,
            storage_devices: storage,
            gpus,
            thermal_sensors: sensors,
        }
    }

    #[test]
    fn ping_reports_runtime_health() {
        let mut engine = RuntimeEngine::with_system_info_provider(MockProvider(Ok(sample())));
        let response = engine.dispatch(request(1, "ping-1", "runtime.ping"));
        assert!(response.ok);
        assert_eq!(response.request_id, "ping-1");
        assert_eq!(response.result.unwrap()["alive"], true);
    }

    #[test]
    fn runtime_info_lists_only_implemented_capabilities() {
        let mut engine = RuntimeEngine::with_system_info_provider(MockProvider(Ok(sample())));
        let result = engine.dispatch(request(1, "info-1", "runtime.getInfo")).result.unwrap();
        assert_eq!(result["capabilities"].as_array().unwrap().len(), 3);
        assert_eq!(result["capabilities"][2], "hardware.discover");
    }

    #[test]
    fn hardware_response_contains_full_snapshot_schema() {
        let mut engine = RuntimeEngine::with_system_info_provider(MockProvider(Ok(sample())));
        let result = engine.dispatch(request(1, "hardware-1", "hardware.discover")).result.unwrap();
        assert_eq!(result["schemaVersion"], 1);
        assert_eq!(result["platform"]["osFamily"], "linux");
        assert_eq!(result["processor"]["frequencyMhz"], 3600);
        assert_eq!(result["memory"]["swapTotalBytes"], 0);
        assert_eq!(result["storageDevices"].as_array().unwrap().len(), 0);
        assert_eq!(result["gpus"].as_array().unwrap().len(), 0);
        assert_eq!(result["thermalSensors"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn unsupported_optional_categories_are_null() {
        let mut snapshot = sample();
        snapshot.gpus = None;
        snapshot.thermal_sensors = None;
        let mut engine = RuntimeEngine::with_system_info_provider(MockProvider(Ok(snapshot)));
        let result = engine.dispatch(request(1, "optional", "hardware.discover")).result.unwrap();
        assert!(result["gpus"].is_null());
        assert!(result["thermalSensors"].is_null());
    }

    #[test]
    fn provider_errors_are_preserved() {
        let mut engine = RuntimeEngine::with_system_info_provider(MockProvider(Err(
            DiscoveryError::UnsupportedPlatform
        )));
        let response = engine.dispatch(request(1, "unsupported", "hardware.discover"));
        assert!(!response.ok);
        assert_eq!(response.error.unwrap().code, "UNSUPPORTED_PLATFORM");
    }

    #[test]
    fn unsupported_protocol_versions_are_rejected() {
        let mut engine = RuntimeEngine::with_system_info_provider(MockProvider(Ok(sample())));
        let response = engine.dispatch(request(99, "bad", "runtime.ping"));
        assert!(!response.ok);
        assert_eq!(response.error.unwrap().code, "PROTOCOL_VERSION_UNSUPPORTED");
    }

    #[test]
    fn unknown_methods_are_rejected() {
        let mut engine = RuntimeEngine::with_system_info_provider(MockProvider(Ok(sample())));
        let response = engine.dispatch(request(1, "unknown", "system.info"));
        assert!(!response.ok);
        assert_eq!(response.error.unwrap().code, "METHOD_NOT_FOUND");
    }
}
