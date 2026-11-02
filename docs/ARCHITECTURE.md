# Daedalus Runtime Architecture

## Purpose

Daedalus is a headless Rust runtime. It must not depend on a desktop UI framework, web frontend, marketplace service, or commercial account system for ordinary local operation.

The public runtime is intended to be consumed by multiple independent clients:
- Daedalus Desktop (private)
- Command-line tools
- Headless services
- Future clients that implement the versioned protocol

## Current implementation

The initial workspace contains:

| Path | Responsibility |
|---|---|
| `crates/runtime-protocol` | Versioned request/response data structures |
| `crates/runtime-engine` | Host-side command dispatch |
| `crates/runtime-ipc` | Newline-delimited JSON over stdin/stdout |
| `apps/runtime-daemon` | Standalone executable |

The only implemented methods are:
- `runtime.ping`
- `runtime.getInfo`

System discovery, workflow scheduling, inference, sandboxing, package parsing and cryptographic verification are not implemented in this initial foundation.

## Dependency direction

```text
runtime-daemon
      |
  runtime-ipc
      |
 runtime-engine
      |
runtime-protocol
```

The engine must not depend on the transport. Protocol types are shared contracts, not Electron or frontend types.

## First feature milestone: system discovery

Add system discovery as a focused runtime capability. Start with:
- Operating-system name/version and architecture.
- CPU model and logical processor count.
- Total and available memory, where reliably available.

Keep platform-specific inspection behind a module boundary and normalize results into Daedalus-owned types. Do not expose third-party system-information crate types in the public protocol. Unknown, unavailable and unsupported values must be represented explicitly.

GPU/accelerator detection is a later milestone and should use platform-specific adapters rather than pretending one library provides complete hardware coverage.

## Cross-platform support

Windows, Linux and macOS are target platforms. A successful cross-compilation is not sufficient evidence of runtime support; CI should build and test on native runners for each target. Platform-specific fields must document availability and limitations.

## Security and privacy

- Keep runtime operation local-first and offline-capable.
- Write protocol responses to stdout only; diagnostics go to stderr.
- Bound request size and validate all requests before dispatch.
- Do not collect serial numbers, persistent hardware identifiers or unrelated personal data for ordinary hardware discovery.
- Treat caller-provided package bytes and workflow definitions as untrusted input.
- Do not claim FIPS validation without an identified validated module and deployment boundary.

## Future architecture (not implemented)

Potential future crates include:
- `system-discovery`
- `workflow-core`
- `workflow-engine`
- `runtime-inference`
- `runtime-sandbox`
- `runtime-crypto`
- `dea-format`
- A headless CLI

Add crates only when concrete functionality and independent testing needs justify them.
