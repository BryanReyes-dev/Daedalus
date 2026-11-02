//! Host-side runtime command dispatcher.
//!
//! This crate deliberately has no Electron, Node.js, Tauri, or transport
//! dependency. New runtime capabilities should be implemented here (or in a
//! focused runtime crate) and exposed through the versioned protocol.

use daedalus_runtime_protocol::{
    RuntimeRequest, RuntimeResponse, CURRENT_PROTOCOL_VERSION,
};
use serde_json::json;

#[derive(Debug, Default)]
pub struct RuntimeEngine;

impl RuntimeEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn dispatch(&self, request: RuntimeRequest) -> RuntimeResponse {
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
                        "runtime.getInfo"
                    ]
                }),
            ),
            method => RuntimeResponse::failure(
                request_id,
                "METHOD_NOT_FOUND",
                format!("Runtime method '{method}' is not available."),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(protocol_version: u16, request_id: &str, method: &str) -> RuntimeRequest {
        RuntimeRequest {
            protocol_version,
            request_id: request_id.to_owned(),
            method: method.to_owned(),
            params: json!({}),
        }
    }

    #[test]
    fn ping_reports_runtime_health() {
        let response = RuntimeEngine::new().dispatch(request(1, "ping-1", "runtime.ping"));

        assert!(response.ok);
        assert_eq!(response.request_id, "ping-1");
        assert_eq!(response.result.expect("ping result")["alive"], true);
    }

    #[test]
    fn runtime_info_reports_only_implemented_capabilities() {
        let response =
            RuntimeEngine::new().dispatch(request(1, "info-1", "runtime.getInfo"));

        let result = response.result.expect("runtime info result");
        assert_eq!(result["name"], "Daedalus Runtime");
        assert_eq!(result["status"], "ready");
        assert_eq!(result["capabilities"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn unsupported_protocol_versions_are_rejected() {
        let response = RuntimeEngine::new().dispatch(request(99, "bad-1", "runtime.ping"));

        assert!(!response.ok);
        assert_eq!(
            response.error.expect("protocol error").code,
            "PROTOCOL_VERSION_UNSUPPORTED"
        );
    }

    #[test]
    fn unknown_methods_are_rejected() {
        let response =
            RuntimeEngine::new().dispatch(request(1, "unknown-1", "system.info"));

        assert!(!response.ok);
        assert_eq!(
            response.error.expect("method error").code,
            "METHOD_NOT_FOUND"
        );
    }
}
