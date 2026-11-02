//! Host-side runtime command dispatcher.
//!
//! This crate deliberately has no Electron, Node.js, Tauri, or transport
//! dependency. Runtime capabilities are exposed through the versioned
//! protocol and implemented behind provider interfaces.

use daedalus_runtime_protocol::{RuntimeRequest, RuntimeResponse, CURRENT_PROTOCOL_VERSION};
use daedalus_system_discovery::SysinfoProvider;
use daedalus_system_info::{SystemInfoProvider, SystemSnapshot};
use serde_json::{json, Value};

pub struct RuntimeEngine {
    system_info_provider: Box<dyn SystemInfoProvider>,
}

impl RuntimeEngine {
    /// Create the host runtime with the default operating-system provider.
    pub fn new() -> Self {
        Self::with_system_info_provider(SysinfoProvider::new())
    }

    /// Construct the runtime with a caller-supplied provider.
    ///
    /// This injection point allows embedded targets or applications to provide
    /// their own board-support-package implementation without changing the
    /// request protocol or engine dispatch logic.
    pub fn with_system_info_provider<P>(provider: P) -> Self
    where
        P: SystemInfoProvider + 'static,
    {
        Self {
            system_info_provider: Box::new(provider),
        }
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
                json!({
                    "alive": true,
                    "protocolVersion": CURRENT_PROTOCOL_VERSION
                }),
            ),
            "runtime.getInfo" => RuntimeResponse::success(
                request_id,
                json!({
                    "name": "Daedalus Runtime",
                    "version": env!("CARGO_PKG_VERSION"),
                    "status": "ready",
                    "protocolVersion": CURRENT_PROTOCOL_VERSION,
                    "capabilities": [
                        "runtime.ping",
                        "runtime.getInfo",
                        "hardware.discover"
                    ]
                }),
            ),
            "hardware.discover" => match self.system_info_provider.discover() {
                Ok(snapshot) => {
                    RuntimeResponse::success(request_id, system_snapshot_to_value(&snapshot))
                }
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
    fn default() -> Self {
        Self::new()
    }
}

fn system_snapshot_to_value(snapshot: &SystemSnapshot) -> Value {
    let processor = snapshot
        .processor
        .as_ref()
        .map(|processor| {
            json!({
                "model": processor.model.as_ref().map(|value| value.as_str()),
                "vendor": processor.vendor.as_ref().map(|value| value.as_str()),
                "logicalProcessors": processor.logical_processors,
                "physicalCores": processor.physical_cores
            })
        })
        .unwrap_or(Value::Null);

    let memory = snapshot
        .memory
        .map(|memory| {
            json!({
                "totalBytes": memory.total_bytes,
                "availableBytes": memory.available_bytes
            })
        })
        .unwrap_or(Value::Null);

    json!({
        "schemaVersion": snapshot.schema_version,
        "status": snapshot.status.as_str(),
        "platform": {
            "osFamily": snapshot.platform.os_family.as_str(),
            "osName": snapshot.platform.os_name.as_ref().map(|value| value.as_str()),
            "osVersion": snapshot.platform.os_version.as_ref().map(|value| value.as_str()),
            "kernelVersion": snapshot.platform.kernel_version.as_ref().map(|value| value.as_str()),
            "architecture": snapshot.platform.architecture.as_str(),
            "architectureName": snapshot.platform.architecture_name.as_ref().map(|value| value.as_str())
        },
        "processor": processor,
        "memory": memory
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use daedalus_system_info::{
        Architecture, DiscoveryError, DiscoveryStatus, FixedText, MemoryInfo, OsFamily,
        PlatformInfo, ProcessorInfo,
    };

    struct MockProvider {
        result: Result<SystemSnapshot, DiscoveryError>,
    }

    impl SystemInfoProvider for MockProvider {
        fn discover(&mut self) -> Result<SystemSnapshot, DiscoveryError> {
            self.result.clone()
        }
    }

    fn request(protocol_version: u16, request_id: &str, method: &str) -> RuntimeRequest {
        RuntimeRequest {
            protocol_version,
            request_id: request_id.to_owned(),
            method: method.to_owned(),
            params: json!({}),
        }
    }

    fn sample_snapshot() -> SystemSnapshot {
        let mut os_name = FixedText::<96>::new();
        os_name.push_str("Test OS").expect("test text fits");

        let mut architecture_name = FixedText::<96>::new();
        architecture_name
            .push_str("x86_64")
            .expect("test architecture fits");

        let mut processor_model = FixedText::<96>::new();
        processor_model
            .push_str("Test CPU")
            .expect("test CPU name fits");

        SystemSnapshot {
            schema_version: 1,
            status: DiscoveryStatus::Complete,
            platform: PlatformInfo {
                os_family: OsFamily::Linux,
                os_name: Some(os_name),
                os_version: None,
                kernel_version: None,
                architecture: Architecture::X86_64,
                architecture_name: Some(architecture_name),
            },
            processor: Some(ProcessorInfo {
                model: Some(processor_model),
                vendor: None,
                logical_processors: Some(8),
                physical_cores: Some(4),
            }),
            memory: Some(MemoryInfo {
                total_bytes: Some(16 * 1024 * 1024 * 1024),
                available_bytes: Some(8 * 1024 * 1024 * 1024),
            }),
        }
    }

    #[test]
    fn ping_reports_runtime_health() {
        let mut engine = RuntimeEngine::with_system_info_provider(MockProvider {
            result: Ok(sample_snapshot()),
        });
        let response = engine.dispatch(request(1, "ping-1", "runtime.ping"));

        assert!(response.ok);
        assert_eq!(response.request_id, "ping-1");
        assert_eq!(response.result.expect("ping result")["alive"], true);
    }

    #[test]
    fn runtime_info_lists_only_implemented_capabilities() {
        let mut engine = RuntimeEngine::with_system_info_provider(MockProvider {
            result: Ok(sample_snapshot()),
        });
        let response = engine.dispatch(request(1, "info-1", "runtime.getInfo"));

        let result = response.result.expect("runtime info result");
        assert_eq!(result["name"], "Daedalus Runtime");
        assert_eq!(result["status"], "ready");
        assert_eq!(result["capabilities"].as_array().unwrap().len(), 3);
        assert_eq!(result["capabilities"][2], "hardware.discover");
    }

    #[test]
    fn hardware_discover_returns_normalized_host_fields() {
        let mut engine = RuntimeEngine::with_system_info_provider(MockProvider {
            result: Ok(sample_snapshot()),
        });
        let response = engine.dispatch(request(1, "hardware-1", "hardware.discover"));

        assert!(response.ok);
        let result = response.result.expect("hardware result");
        assert_eq!(result["schemaVersion"], 1);
        assert_eq!(result["status"], "complete");
        assert_eq!(result["platform"]["osFamily"], "linux");
        assert_eq!(result["platform"]["architecture"], "x86_64");
        assert_eq!(result["platform"]["osName"], "Test OS");
        assert_eq!(result["processor"]["logicalProcessors"], 8);
        assert_eq!(result["processor"]["physicalCores"], 4);
        assert_eq!(result["memory"]["totalBytes"], 16_u64 * 1024 * 1024 * 1024);
    }

    #[test]
    fn hardware_discover_returns_explicit_provider_errors() {
        let mut engine = RuntimeEngine::with_system_info_provider(MockProvider {
            result: Err(DiscoveryError::UnsupportedPlatform),
        });
        let response = engine.dispatch(request(1, "hardware-unsupported", "hardware.discover"));

        assert!(!response.ok);
        assert_eq!(
            response.error.expect("provider error").code,
            "UNSUPPORTED_PLATFORM"
        );
    }

    #[test]
    fn unsupported_protocol_versions_are_rejected() {
        let mut engine = RuntimeEngine::with_system_info_provider(MockProvider {
            result: Ok(sample_snapshot()),
        });
        let response = engine.dispatch(request(99, "bad-1", "runtime.ping"));

        assert!(!response.ok);
        assert_eq!(
            response.error.expect("protocol error").code,
            "PROTOCOL_VERSION_UNSUPPORTED"
        );
    }

    #[test]
    fn unknown_methods_are_rejected() {
        let mut engine = RuntimeEngine::with_system_info_provider(MockProvider {
            result: Ok(sample_snapshot()),
        });
        let response = engine.dispatch(request(1, "unknown-1", "system.info"));

        assert!(!response.ok);
        assert_eq!(
            response.error.expect("method error").code,
            "METHOD_NOT_FOUND"
        );
    }
}
