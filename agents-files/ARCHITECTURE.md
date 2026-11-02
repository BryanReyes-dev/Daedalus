# Daedalus — Architecture

## Document authority and status

This is the durable architecture record for the current Rust runtime repository. It describes the code that exists and the boundaries future work must preserve; it does not silently promote roadmap items to implemented functionality.

- [Agents_Context.md](./Agents_Context.md) contains current status and time-sensitive verification evidence.
- [AGENTS.md](./AGENTS.md) contains coding-agent rules.
- [CLAUDE.md](./CLAUDE.md) is the Claude Code entry point.
- [../README.md](../README.md) is the human-facing project and developer guide.
- [../docs/SYSTEM_DISCOVERY.md](../docs/SYSTEM_DISCOVERY.md) is the detailed hardware-discovery API reference.

Last reviewed: 2026-11-02. The current workspace was reported by the project author as passing Windows 11 formatting, compilation, tests, and CLI smoke checks. Native testing on other platforms has not been reported as complete.

## 1. Product boundaries and repository state

Daedalus is intended to be a local-first, headless runtime for hardware discovery and future AI/workflow execution. The Rust runtime should be usable by CLI, headless, desktop, and future edge clients without requiring a web UI or marketplace connection for ordinary local operations.

The current repository contains the Rust workspace described below. The ecosystem is intended to keep separate product responsibilities:

| Product area | Responsibility |
|---|---|
| Daedalus Rust runtime | Portable contracts, host adapters, protocol, runtime dispatch, CLI, and daemon |
| Daedalus Desktop | Private desktop interface and application experience, including the Electron shell and visual workflow UI |
| Daedalus Marketplace | Private hosted storefront, creator services, distribution, commercial entitlements, payments, and payouts |

The repository is currently private. Its checked-in LICENSE remains proprietary / All Rights Reserved. Any later public release or Apache-2.0 licensing change requires a separate, explicit licensing and publication decision, the correct license text, contribution terms, and repository visibility review. A README statement or an architecture goal does not grant a license.

The Rust runtime must not absorb proprietary desktop UI or marketplace business logic. Interfaces to those systems should be explicit and should not make marketplace availability a prerequisite for local runtime operation.

## 2. Current workspace

The Cargo workspace is defined in the root `Cargo.toml`.

| Path / package | Responsibility | Runtime boundary |
|---|---|---|
| `crates/daedalus-system-info` / `daedalus-system-info` | Portable system-information types and `SystemInfoProvider` trait | `no_std`, no heap allocation |
| `crates/daedalus-system-discovery` / `daedalus-system-discovery` | Host discovery adapter using `sysinfo` | `std`, host APIs |
| `crates/runtime-protocol` / `daedalus-runtime-protocol` | JSON request and response structures | `std`, serde |
| `crates/runtime-engine` / `daedalus-runtime-engine` | Method dispatch and provider orchestration | `std` |
| `crates/runtime-ipc` / `daedalus-runtime-ipc` | Newline-delimited JSON stdio transport | `std` |
| `apps/daedalus-cli` / `daedalus-cli` | In-process command-line inspector; binary name `daedalus` | `std` |
| `apps/runtime-daemon` / `daedalus-runtime` | Standalone protocol process; binary name `daedalus-runtime` | `std` |
| `agents-files/` | Agent operating rules, working context, and architecture authority | Documentation |
| `docs/` | Focused technical references, including system discovery | Documentation |

### Dependency direction

```text
daedalus-cli --------------------------+
                                       |
daedalus-runtime -> runtime-ipc -> runtime-engine
                                       |
                              SystemInfoProvider
                                       |
                         daedalus-system-discovery
                                       |
                           daedalus-system-info
```

The runtime engine receives a `SystemInfoProvider` implementation and dispatches requests. The default constructor uses `SysinfoProvider`; tests and future board-specific targets can inject another provider. The portable contract does not depend on the host adapter.

## 3. Portable system-information contract

`daedalus-system-info` is the stable boundary between hardware providers and runtime consumers.

It is `#![no_std]`, uses fixed-capacity heapless strings/vectors, and makes no operating-system calls. The current contract contains normalized values for:

- OS family and name/version/kernel information.
- CPU architecture, model/vendor, logical processor count, physical core count, and reported frequency.
- RAM and swap total, available, and used byte counts.
- Product and motherboard manufacturer/model/version metadata.
- Storage kind, filesystem, capacity/free space, removable state, and read-only state.
- GPU vendor/model, memory, and utilization where provided.
- Thermal sensor label, current temperature, maximum, and critical temperature where provided.

The current fixed collection bounds are 16 storage entries, 8 GPU entries, and 32 thermal sensors. Text is held in fixed-capacity 96-byte heapless strings. Providers should preserve optionality and bounds rather than silently substituting invented values.

### Discovery status

A snapshot is marked `complete` when its baseline OS label, architecture label, logical processor count, and total RAM value are present. Otherwise it is `partial`. Complete does not mean every optional hardware category was detected.

For optional device collections:

- `None` serializes as JSON `null`: the category could not be queried.
- `Some(empty)` serializes as `[]`: the query ran and found no entries.
- A populated vector serializes as its bounded array of entries.

Unavailable scalar readings serialize as JSON `null`. Byte counts are bytes, CPU frequency is MHz, GPU utilization is a percentage, and thermal readings are degrees Celsius.

## 4. Host discovery adapter

`daedalus-system-discovery` is the `std` host adapter. It uses `sysinfo 0.40.1` and converts upstream readings into Daedalus-owned contract types.

The current adapter queries:
- CPU model/vendor, logical processor count, physical core count, and reported frequency.
- RAM and swap totals and usage.
- Product and motherboard model metadata when exposed.
- Storage entries and volume information without returning mount paths.
- GPU inventory where the upstream API and platform support it.
- Exposed thermal components and readings.

The adapter intentionally does not request process/user lists, host names, network interfaces, IP/MAC addresses, disk mount paths, serial numbers, UUIDs, asset tags, PCI addresses, or persistent hardware identifiers.

### Upstream coverage versus tested coverage

The upstream `sysinfo` support matrix documents Windows, Linux, macOS, Android, iOS, FreeBSD, NetBSD, and Raspberry Pi-class Linux systems. Individual information categories have narrower platform support and depend on OS permissions, kernel APIs, drivers, virtualization, and hardware exposure.

GPU inventory is available through the upstream API on Windows, Linux, and macOS. Product/motherboard and thermal information have platform gaps. Containers, WSL, VMs, and sandboxes can expose incomplete or guest-scoped results.

These are upstream capability statements, not proof that this repository has been natively tested on every listed platform. As of this review, Windows 11 is the only platform with reported workspace and CLI validation. Other targets require native validation before being described as tested or supported by this project.

## 5. Runtime protocol and method dispatch

The wire envelope uses protocol version 1:

```json
{
  "protocolVersion": 1,
  "requestId": "hardware-001",
  "method": "hardware.discover",
  "params": {}
}
```

The current methods are:

| Method | Behavior |
|---|---|
| `runtime.ping` | Reports that the in-process runtime dispatcher is responding |
| `runtime.getInfo` | Returns runtime identity, version, protocol version, and implemented capabilities |
| `hardware.discover` | Returns a hardware/system snapshot from the configured provider |

The transport protocol version is independent of the hardware snapshot `schemaVersion`. A future incompatible wire change must update the protocol version and corresponding tests/documentation. Snapshot-shape evolution must be handled deliberately and documented separately.

Current failure codes include `INVALID_REQUEST`, `PROTOCOL_VERSION_UNSUPPORTED`, `METHOD_NOT_FOUND`, `UNSUPPORTED_PLATFORM`, `PROVIDER_INITIALIZATION_FAILED`, and `SYSTEM_DATA_UNAVAILABLE`.

## 6. CLI and daemon

The `daedalus` CLI directly creates and dispatches a `RuntimeEngine` in its own process. Its current commands are `hardware`, `info`, `ping`, `help`, `--help`, and `--version`. Runtime command responses are pretty-printed JSON. The CLI does not start or contact the separate daemon.

The `daedalus-runtime` binary uses `runtime-ipc` to process newline-delimited JSON on stdin and write one response per non-empty request line to stdout. It can process multiple requests in one process. Malformed JSON yields an `INVALID_REQUEST` response and does not stop subsequent input. Stdout is reserved for protocol output; diagnostics go to stderr.

## 7. Security and privacy boundaries

- Discovery is local capability inspection, not telemetry collection.
- The default provider omits stable or sensitive identifiers and unrelated host/user data.
- Device information is machine-specific. Clients must obtain owner consent before sending it off-device.
- Keep bounded strings and bounded device collections; do not allow host data to drive unbounded allocations in the portable contract.
- Treat protocol input and future package/workflow input as untrusted.
- Keep ordinary discovery and runtime dispatch functional without a network connection.
- Keep marketplace authorization, payment secrets, creator data, and DRM/business rules outside the core runtime.
- Do not describe a Rust implementation or cryptographic dependency as FIPS-validated. Any future compliance claim must identify the validated module, exact operating environment, and deployment boundary.

## 8. Embedded boundary

The portable `daedalus-system-info` crate can be compiled for a `no_std` target and exposes a provider trait that firmware can implement using a board-support package, HAL, or RTOS.

The current host adapter, engine, protocol, JSON serialization, CLI, and daemon use `std`. This repository does not currently contain a complete bare-metal runtime or a board-specific embedded provider. A microcontroller integration needs its own provider, target-specific resource model, transport/in-process profile, and physical-board tests.

## 9. Testing and evidence

The workspace contains unit tests in the portable contract, host adapter, runtime protocol, runtime engine, and stdio transport crates.

The project author reported the following successful Windows 11 checks on 2026-11-02:
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace -- --test-threads=1`
- CLI smoke checks for `hardware`, `info`, `ping`, `--help`, and `--version`

This is user-reported local evidence. No result in this document implies that tests were run by GitHub Actions. Native tests on Linux, macOS, Android/iOS, BSD, and Raspberry Pi hardware are still pending. Cross-compilation and upstream crate support do not prove native runtime behavior.

## 10. Future architecture (not implemented)

The intended direction may include the following capabilities. They are plans, not existing APIs or guarantees:

1. **Cross-platform validation:** native test coverage and CI for agreed target OS/version/hardware combinations.
2. **Richer capability discovery:** GPU driver versions, CUDA/ROCm/Metal/Vulkan/OpenCL capability queries, accelerator memory/topology, battery, power, and firmware information where safe and available.
3. **Workflow runtime:** portable workflow/DAG definitions, validation, deterministic state/lifecycle, execution scheduling, cancellation, resource budgets, and observability.
4. **Local inference:** model metadata/loading and in-process Rust inference backends, with optional platform-specific acceleration. Local/offline operation remains the design goal.
5. **Sandboxed components:** a capability-restricted WASI Component Model execution path, likely using a Wasmtime-based runtime after threat modeling and resource isolation design.
6. **Package and signature format:** a documented `.dea` package format, bounded parsing, integrity checks, signatures, and caller-controlled trust roots. No package parser or signature verification is implemented today.
7. **Developer and platform integrations:** stable interfaces and SDKs for desktop/headless clients and potential Rust, TypeScript, Python, or Go integrations.
8. **Embedded and edge profiles:** board-specific providers, HAL/BSP/RTOS integration, in-process runtime profiles, and target-specific transports.

Each item requires an explicit design, threat model where applicable, implementation, tests, and updated docs before it can be described as complete.
