//! Versioned wire contract shared by runtime transports and clients.
//!
//! The first transport is newline-delimited JSON over the runtime process's
//! stdin/stdout. Keep stdout reserved for protocol messages; diagnostics belong
//! on stderr.

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CURRENT_PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeRequest {
    pub protocol_version: u16,
    pub request_id: String,
    pub method: String,
    #[serde(default = "empty_params")]
    pub params: Value,
}

fn empty_params() -> Value {
    Value::Object(Default::default())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeResponse {
    pub protocol_version: u16,
    pub request_id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RuntimeError>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeError {
    pub code: String,
    pub message: String,
}

impl RuntimeResponse {
    pub fn success(request_id: String, result: Value) -> Self {
        Self {
            protocol_version: CURRENT_PROTOCOL_VERSION,
            request_id,
            ok: true,
            result: Some(result),
            error: None,
        }
    }

    pub fn failure(
        request_id: String,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            protocol_version: CURRENT_PROTOCOL_VERSION,
            request_id,
            ok: false,
            result: None,
            error: Some(RuntimeError {
                code: code.into(),
                message: message.into(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_uses_camel_case_wire_fields() {
        let request: RuntimeRequest = serde_json::from_str(
            r#"{"protocolVersion":1,"requestId":"abc","method":"runtime.ping"}"#,
        )
        .expect("request should deserialize");

        assert_eq!(request.protocol_version, CURRENT_PROTOCOL_VERSION);
        assert_eq!(request.request_id, "abc");
        assert_eq!(request.params, Value::Object(Default::default()));
    }

    #[test]
    fn response_omits_the_unused_outcome_field() {
        let response = RuntimeResponse::success(
            "abc".to_owned(),
            serde_json::json!({ "alive": true }),
        );

        let value = serde_json::to_value(response).expect("response should serialize");

        assert_eq!(value["protocolVersion"], CURRENT_PROTOCOL_VERSION);
        assert_eq!(value["requestId"], "abc");
        assert_eq!(value["ok"], true);
        assert_eq!(value["result"]["alive"], true);
        assert!(value.get("error").is_none());
    }
}
