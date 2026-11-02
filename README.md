# Daedalus

**Daedalus is a local-first, headless runtime foundation written in Rust, with a long-term goal of supporting offline AI workflows and inference.**

The current implementation provides a portable system-information contract, a host hardware-discovery adapter, a versioned JSON protocol, a command-line inspector, and a standalone runtime daemon. It is designed so local inspection does not depend on the desktop interface or hosted marketplace.

> **Implementation status:** The Rust runtime foundation and first system-discovery milestone are merged into `main`. The workspace and CLI have been validated on Windows 11 by the project author. Native validation on other operating systems, GitHub Actions, workflow execution, model inference, package handling, and sandboxing remain future work.

**Repository:** [BryanReyes-dev/Daedalus](https://github.com/BryanReyes-dev/Daedalus) · **Current version:** `0.2.0` · **Protocol:** `1`

## Table of Contents

- [Overview](#overview)
  - [Project goals](#project-goals)
  - [Current implementation](#current-implementation)
  - [Technology and workspace](#technology-and-workspace)
  - [Repository boundaries](#repository-boundaries)
- [Architecture](#architecture)
  - [Request flow](#request-flow)
  - [Design decisions](#design-decisions)
  - [Portable system-information contract](#portable-system-information-contract)
  - [Host discovery coverage](#host-discovery-coverage)
- [Getting Started](#getting-started)
  - [Requirements](#requirements)
  - [Clone and build](#clone-and-build)
- [Runtime Usage](#runtime-usage)
  - [CLI reference](#cli-reference)
  - [Hardware discovery output](#hardware-discovery-output)
  - [Runtime daemon](#runtime-daemon)
  - [JSON protocol](#json-protocol)
  - [Runtime error codes](#runtime-error-codes)
- [Development](#development)
  - [Workspace layout](#workspace-layout)
  - [Crate responsibilities](#crate-responsibilities)
  - [Formatting and builds](#formatting-and-builds)
- [Testing](#testing)
  - [Workspace test suite](#workspace-test-suite)
  - [Focused crate tests](#focused-crate-tests)
  - [CLI and daemon smoke tests](#cli-and-daemon-smoke-tests)
  - [Checking the no_std contract](#checking-the-no_std-contract)
  - [Platform validation status](#platform-validation-status)
- [Security and Privacy](#security-and-privacy)
  - [Discovery boundaries](#discovery-boundaries)
  - [Embedded boundary](#embedded-boundary)
- [Future Plans](#future-plans)
  - [Phase 1 — cross-platform validation](#phase-1--cross-platform-validation)
  - [Phase 2 — richer hardware capability discovery](#phase-2--richer-hardware-capability-discovery)
  - [Phase 3 — workflow runtime](#phase-3--workflow-runtime)
  - [Phase 4 — local inference](#phase-4--local-inference)
  - [Phase 5 — sandboxing and package format](#phase-5--sandboxing-and-package-format)
  - [Phase 6 — developer, desktop, and embedded integrations](#phase-6--developer-desktop-and-embedded-integrations)
- [Agent and Project Documentation](#agent-and-project-documentation)
- [Licensing and Release Status](#licensing-and-release-status)

## Overview

Daedalus is being developed as a local-first runtime that can serve desktop, CLI, and headless applications without tying execution to a particular UI framework. The current repository is intentionally much smaller than the long-term product vision: it establishes the Rust workspace, provider boundary, JSON protocol, CLI, daemon, and initial system-discovery capabilities.

### Project goals

The long-term design is guided by several constraints:

- **Local-first:** ordinary runtime operations should work without a network connection.
- **Headless core:** execution and hardware inspection belong in Rust, not in the desktop UI.
- **Portable contracts:** host-specific APIs should remain behind provider adapters.
- **Explicit capability reporting:** unavailable, unsupported, empty, and detected results must not be conflated.
- **Privacy by default:** discover only machine information needed for capability inspection; avoid persistent device identity and unrelated user data.
- **Independent interfaces:** CLI, daemon, desktop, and future embedded clients should use well-defined runtime contracts.
- **Security-conscious execution:** future workflow components and package inputs must be treated as untrusted and isolated by explicit capability boundaries.

### Current implementation

The following capabilities exist in the current `main` branch:

- A `no_std`, no-heap, fixed-capacity system-information contract.
- Normalized operating-system family and CPU architecture values.
- CPU model/vendor, logical processor count, physical core count, and reported frequency.
- RAM and swap total, available, and used byte counts.
- Product and motherboard manufacturer/model/version metadata where exposed.
- Storage inventory with device kind, filesystem, capacity/free space, removable, and read-only state.
- GPU vendor/model, memory, and utilization where exposed by the OS and driver.
- Thermal sensor labels and current/maximum/critical readings where exposed.
- Versioned JSON request/response structures and three implemented runtime methods.
- An in-process `daedalus` CLI.
- A separate `daedalus-runtime` newline-delimited JSON daemon.
- Unit tests across the portable contract, discovery adapter, protocol, engine, and IPC crates.

The host adapter does not enumerate processes, user accounts, host names, network interfaces, IP addresses, MAC addresses, disk mount paths, hardware serial numbers, product UUIDs, asset tags, or PCI bus addresses.

### Technology and workspace

| Area | Technology |
|---|---|
| Language | Rust, workspace edition 2021 |
| Workspace/dependencies | Cargo, committed `Cargo.lock` |
| Portable contract | `no_std`, `heapless` fixed-capacity strings/vectors |
| Host discovery | `sysinfo 0.40.1` |
| Runtime protocol | Serde and JSON |
| Transport | Newline-delimited JSON over stdin/stdout |
| Executables | `daedalus` CLI and `daedalus-runtime` daemon |
| Tests | Rust unit tests and Cargo workspace test suite |

The current `sysinfo` dependency requires Rust 1.95 or newer.

### Repository boundaries

| Product area | Responsibility |
|---|---|
| Daedalus Rust runtime | Portable types, host adapters, protocol, runtime dispatch, CLI, daemon, and runtime specifications |
| Daedalus Desktop | Separate private desktop experience, Electron shell, visual workflow UI, and application integrations |
| Daedalus Marketplace | Separate private storefront, creator services, artifact distribution, commercial entitlements, payments, and payouts |

The repository is publicly visible, but it remains under its checked-in proprietary license. Public visibility does not change the licensing terms. See [Licensing and Release Status](#licensing-and-release-status).

## Architecture

The runtime is separated into portable contracts, host-specific discovery, request dispatch, transport, and executable entry points.

### Request flow

The CLI and daemon share the same runtime engine but use different entry paths:

```text
daedalus CLI ----------------------------+
                                         |
daedalus-runtime -> runtime-ipc -> runtime-engine
                                         |
                                 SystemInfoProvider
                                         |
                          daedalus-system-discovery
                                         |
                             daedalus-system-info
```

- The CLI constructs and calls `RuntimeEngine` directly inside its process.
- The daemon reads protocol requests from stdin and writes responses to stdout through `runtime-ipc`.
- The engine dispatches methods and invokes an injected `SystemInfoProvider`.
- The default host provider uses `sysinfo`; tests and future platform-specific providers can implement the same trait.
- The `no_std` contract does not depend on the host adapter.

See [agents-files/ARCHITECTURE.md](./agents-files/ARCHITECTURE.md) for the durable architecture record and [docs/SYSTEM_DISCOVERY.md](./docs/SYSTEM_DISCOVERY.md) for the hardware-discovery contract.

### Design decisions

#### Why keep `daedalus-system-info` no_std and allocation-free?

The portable contract is deliberately separated from host discovery and JSON transport. It uses fixed-capacity `heapless` strings and vectors, so representing a snapshot does not require a global allocator or operating-system APIs. That keeps the data model usable by a future firmware, RTOS, or board-specific provider that may not have a general-purpose heap.

The host adapter and the current runtime/transport layers are separate `std` crates. `sysinfo` is used only by the host-discovery adapter; JSON serialization is performed by the runtime crates. Keeping those dependencies out of `daedalus-system-info` prevents host and transport requirements from becoming requirements of the portable data contract.

This boundary is an embedded-enabling design decision, not a claim that the current daemon or CLI runs on a microcontroller. A board-specific provider, target profile, transport, and native hardware tests are still required. The documented `thumbv7em-none-eabihf` check is a suggested future validation step; it has not been reported as run.

#### Why newline-delimited JSON over stdin/stdout?

The daemon uses one JSON request per input line and one JSON response per output line. This is intentionally a small local process protocol:

- It does not need an HTTP listener, an open TCP port, port allocation, or port-collision handling.
- It avoids requiring a network firewall rule solely for communication with a locally supervised child process.
- It gives a future desktop host a straightforward way to spawn the runtime, pipe requests/responses, and manage the child process lifecycle.
- It keeps the transport easy to inspect with shell tools and simple to exercise in tests.

The current repository implements the stdio daemon; it does not yet implement desktop-side process spawning or a remote HTTP/gRPC service. A network transport could be added later if multi-client or remote access becomes a real requirement.

#### Why run host-discovery tests serially?

The recommended workspace command is:

```bash
cargo test --workspace -- --test-threads=1
```

System-information tests inspect live host state rather than a fully isolated fixture. The upstream `sysinfo` project specifically recommends limiting parallel test execution because its system-inspection tests are more likely to succeed with fewer concurrent test threads. Serial execution follows that guidance and reduces overlapping host queries. This README does not assert a specific WMI race or kernel-handle bug in Daedalus.

### Portable system-information contract

The `daedalus-system-info` crate contains no OS calls and does not require a heap allocator. It defines fixed-capacity strings and collections, normalized OS/architecture types, snapshot structures, discovery status, and the `SystemInfoProvider` trait.

The current capacity bounds are 16 storage entries, 8 GPUs, and 32 thermal sensors. Text fields use fixed-capacity heapless strings.

A snapshot is `complete` when baseline OS label, architecture label, logical processor count, and total RAM are present. Otherwise it is `partial`. A complete snapshot does not guarantee all optional categories are populated.

For `storageDevices`, `gpus`, and `thermalSensors`:
- `null` means the category could not be queried.
- `[]` means the query succeeded but returned no entries.
- A populated array contains the entries returned, up to the contract's fixed limit.

### Host discovery coverage

The host adapter uses [sysinfo 0.40.1](https://docs.rs/sysinfo/0.40.1/sysinfo/). Upstream documents support for Windows, Linux, macOS, Android, iOS, FreeBSD, NetBSD, and Raspberry Pi-class Linux systems. Individual data categories have narrower support and depend on the OS, permissions, drivers, virtualization, and exposed hardware.

- GPU inventory is available through the upstream API on Windows, Linux, and macOS.
- Product and motherboard metadata are not implemented on NetBSD.
- Thermal component enumeration is not implemented on NetBSD and may be unavailable in virtualized or restricted environments.
- Android versions and permissions can restrict some CPU readings.
- macOS uses shared GPU/system memory, so dedicated VRAM fields may be null.
- Storage inventory represents OS-exposed volumes and does not promise physical-drive topology or controller/RAID details.

**Upstream support is not the same as project validation.** Windows 11 is the only platform with reported workspace and CLI validation so far. See [Platform validation status](#platform-validation-status).

## Getting Started

### Requirements

- Rust 1.95 or newer. This is the minimum supported Rust version (MSRV) of the pinned `sysinfo 0.40.1` dependency, not a claim that Rust 1.95 is the latest stable release. See the [sysinfo 0.40.1 crate documentation](https://docs.rs/crate/sysinfo/0.40.1) for its published MSRV.
- Cargo (installed with the Rust toolchain).
- Git.
- A host OS supported by the current discovery backend for live hardware information.

Check the toolchain:

```bash
rustc --version
cargo --version
```

### Clone and build

Clone the publicly visible repository:

```bash
git clone https://github.com/BryanReyes-dev/Daedalus.git
cd Daedalus
```

Build the workspace:

```bash
cargo build --workspace
```

Run a first discovery:

```bash
cargo run -p daedalus-cli -- hardware
```

The command prints a pretty-printed JSON response. It does not upload, persist, or transmit the snapshot.

## Runtime Usage

### CLI reference

The CLI binary is named `daedalus`. During development, invoke it through Cargo from the workspace root:

```bash
cargo run -p daedalus-cli -- <COMMAND>
```

After a release build, the binary is located at:

- Windows: `target/release/daedalus.exe`
- Linux/macOS: `target/release/daedalus`

| Command | Purpose |
|---|---|
| `hardware` | Discover platform, CPU, memory, storage, GPU, and exposed thermal sensors |
| `info` | Print runtime identity, version, protocol version, and implemented methods |
| `ping` | Check the in-process runtime engine |
| `help`, `-h`, `--help` | Show usage |
| `-V`, `--version` | Show CLI version |

Examples:

```bash
cargo run -p daedalus-cli -- hardware
cargo run -p daedalus-cli -- info
cargo run -p daedalus-cli -- ping
cargo run -p daedalus-cli -- --help
cargo run -p daedalus-cli -- --version
```

The CLI accepts one command at a time and currently has no command-specific flags. Runtime results are written to stdout as JSON; usage errors are written to stderr.

Exit codes:

| Code | Meaning |
|---:|---|
| `0` | Command succeeded |
| `1` | Runtime/provider or response-serialization failure |
| `2` | Unknown command or unexpected extra argument |

The `ping` command checks the in-process engine. It does not start or contact the separate daemon.

### Hardware discovery output

A successful `hardware` response uses the standard runtime response envelope. The snapshot is nested under `result`.

The excerpt below preserves core fields from an earlier Windows 11 development-machine capture. It is intentionally partial—not a fabricated full response—and the values are specific to that machine:

```json
{
  "platform": {
    "osFamily": "windows",
    "architecture": "x86_64"
  },
  "processor": {
    "model": "AMD Ryzen 5 5500",
    "logicalProcessors": 12,
    "physicalCores": 6
  },
  "memory": {
    "totalBytes": 16960598016
  }
}
```

To capture a fresh complete response from your own machine:

```bash
cargo run -p daedalus-cli -- hardware > hardware-output.json
```

The redirected file contains machine-specific platform and hardware information. Review it before sharing or committing it; it is not needed by the project test suite.

| Field | Description |
|---|---|
| `schemaVersion` | Hardware snapshot schema version, independent of protocol version |
| `status` | `complete` or `partial` based on baseline OS, architecture, logical CPU, and RAM values |
| `platform` | OS family/name/version, kernel version, normalized and raw CPU architecture |
| `processor` | CPU model/vendor, logical processors, physical cores, and reported frequency |
| `memory` | RAM and swap totals, available, and used values |
| `product` | System manufacturer/model/family/version where available |
| `motherboard` | Board manufacturer/model/version where available |
| `storageDevices` | Device kind, filesystem, capacity/free bytes, removable/read-only state |
| `gpus` | GPU vendor/model, memory, and utilization where available |
| `thermalSensors` | Sensor label and current/maximum/critical temperatures where available |

Units:
- Memory and storage values are bytes.
- CPU frequency is MHz.
- GPU utilization is a percentage.
- Temperatures are degrees Celsius.

Unavailable scalar values are JSON `null`. Optional device arrays distinguish `null` (could not query the category) from `[]` (query succeeded, no entries found). The complete status applies only to baseline platform/CPU/RAM values.

### Runtime daemon

The workspace builds a separate process named `daedalus-runtime`. Unlike the CLI, the daemon reads newline-delimited JSON requests from stdin and writes one JSON response per line to stdout.

Start it:

```bash
cargo run -p daedalus-runtime
```

Send a ping request from Git Bash:

```bash
printf '%s\n' '{"protocolVersion":1,"requestId":"ping-001","method":"runtime.ping","params":{}}' | cargo run -p daedalus-runtime
```

Send a hardware request:

```bash
printf '%s\n' '{"protocolVersion":1,"requestId":"hardware-001","method":"hardware.discover","params":{}}' | cargo run -p daedalus-runtime
```

The daemon accepts multiple input lines in one process. Each non-empty line produces one response line. Malformed JSON produces an `INVALID_REQUEST` response and does not stop subsequent requests. Stdout is reserved for protocol messages; diagnostics go to stderr.

### JSON protocol

Protocol version 1 uses this request envelope:

```json
{
  "protocolVersion": 1,
  "requestId": "hardware-001",
  "method": "hardware.discover",
  "params": {}
}
```

- `protocolVersion` must match the runtime protocol version.
- `requestId` is copied into the response for correlation.
- `method` selects the operation.
- `params` defaults to an empty object; the current methods do not require parameters.

Successful response example:

```json
{
  "protocolVersion": 1,
  "requestId": "ping-001",
  "ok": true,
  "result": {
    "alive": true,
    "protocolVersion": 1
  }
}
```

Failure response example:

```json
{
  "protocolVersion": 1,
  "requestId": "hardware-001",
  "ok": false,
  "error": {
    "code": "UNSUPPORTED_PLATFORM",
    "message": "No system-information provider is available for this target."
  }
}
```

The unused `result` or `error` property is omitted. The request/response protocol version and hardware snapshot schema version are separate.

### Runtime error codes

| Code | Meaning |
|---|---|
| `INVALID_REQUEST` | An input line could not be parsed |
| `PROTOCOL_VERSION_UNSUPPORTED` | Client and runtime protocol versions differ |
| `METHOD_NOT_FOUND` | Requested method is not implemented |
| `UNSUPPORTED_PLATFORM` | No host provider is available for this target |
| `PROVIDER_INITIALIZATION_FAILED` | Host provider could not initialize |
| `SYSTEM_DATA_UNAVAILABLE` | Provider could not read system data |

## Development

Run Cargo commands from the workspace root.

### Workspace layout

```text
apps/
  daedalus-cli/                 # In-process command-line inspector
  runtime-daemon/               # JSON-over-stdio daemon
crates/
  daedalus-system-info/         # no_std portable contract and provider trait
  daedalus-system-discovery/    # std host adapter using sysinfo
  runtime-engine/               # method dispatch and provider orchestration
  runtime-ipc/                  # newline-delimited JSON transport
  runtime-protocol/             # versioned request/response structures
agents-files/
  AGENTS.md                     # agent workflow and constraints
  Agents_Context.md             # current status and evidence
  ARCHITECTURE.md               # durable architecture authority
  CLAUDE.md                     # Claude Code imports
docs/
  SYSTEM_DISCOVERY.md           # discovery API and field semantics
```

### Crate responsibilities

- **`daedalus-system-info`:** normalized bounded types, snapshot structures, and the provider trait. No OS calls, `std`, or heap allocator.
- **`daedalus-system-discovery`:** host-specific discovery through `sysinfo`.
- **`daedalus-runtime-protocol`:** Serde request/response envelope and protocol version.
- **`daedalus-runtime-engine`:** method dispatch, provider injection, and JSON result construction.
- **`daedalus-runtime-ipc`:** line-oriented stdin/stdout server.
- **`daedalus-cli`:** the `daedalus` executable, invoking the engine in-process.
- **`daedalus-runtime`:** the standalone daemon executable.

### Formatting and builds

```bash
cargo fmt --all
cargo fmt --all -- --check
cargo check --workspace
cargo build --workspace
cargo build --workspace --release
```

`Cargo.lock` is committed for reproducible dependency resolution. When dependencies or features change, regenerate and commit the lockfile alongside the relevant Cargo manifest changes.

## Testing

### Workspace test suite

Run the full suite from the workspace root:

```bash
cargo test --workspace -- --test-threads=1
```

Serial execution is recommended for tests that inspect live host system information. The upstream [sysinfo testing guidance](https://docs.rs/crate/sysinfo/0.40.1) also recommends limiting the number of parallel test threads for system-inspection tests. This is a reliability recommendation; it is not a claim that Daedalus has a confirmed WMI race condition.

### Focused crate tests

```bash
cargo test -p daedalus-system-info
cargo test -p daedalus-system-discovery -- --test-threads=1
cargo test -p daedalus-runtime-protocol
cargo test -p daedalus-runtime-engine
cargo test -p daedalus-runtime-ipc
```

### CLI and daemon smoke tests

```bash
cargo run -p daedalus-cli -- hardware
cargo run -p daedalus-cli -- info
cargo run -p daedalus-cli -- ping
cargo run -p daedalus-cli -- --help
cargo run -p daedalus-cli -- --version
printf '%s\n' '{"protocolVersion":1,"requestId":"ping-001","method":"runtime.ping","params":{}}' | cargo run -p daedalus-runtime
```

### Checking the no_std contract

The portable contract can be checked for an embedded target separately:

```bash
rustup target add thumbv7em-none-eabihf
cargo check -p daedalus-system-info --target thumbv7em-none-eabihf
```

This checks only `daedalus-system-info`. It does not make the host adapter, runtime engine, JSON protocol, CLI, or daemon execute on a microcontroller.

### Platform validation status

The project author reported successful validation on Windows 11 on 2026-11-02:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo test --workspace -- --test-threads=1`
- CLI smoke tests for `hardware`, `info`, `ping`, `--help`, and `--version`

This is project-author-reported local evidence, not a GitHub Actions result. The reported development toolchain was Rust 1.99.1 / Cargo 1.99.0; the dependency MSRV remains 1.95. Native runtime testing on Linux, macOS, Android/iOS, BSD, and Raspberry Pi hardware remains pending. Cross-compilation or upstream `sysinfo` support is not proof of correct runtime discovery on a target machine.

GitHub Actions and per-platform CI workflows are deliberately deferred to a later task.

## Security and Privacy

### Discovery boundaries

System discovery is intended for local capability inspection, not telemetry collection. The default provider omits:

- Process lists, executable paths, command-line arguments, and user accounts.
- Host names and network interfaces.
- IP addresses and MAC addresses.
- Disk mount paths.
- Hardware serial numbers, product UUIDs, asset tags, persistent device identifiers, and PCI bus addresses.

Returned system information is still machine-specific. Any client that transmits a snapshot outside the device must obtain the machine owner's consent.

Treat all future workflow definitions, package files, models, and component inputs as untrusted. Design explicit parsing limits, trust roots, permissions, and execution boundaries before implementing those features.

### Embedded boundary

The portable `daedalus-system-info` contract is `no_std` and exposes `SystemInfoProvider` for a firmware provider implemented with a target's HAL/BSP/RTOS APIs.

The current host adapter, runtime engine, JSON process, CLI, and daemon all use `std`. This project does not yet provide a complete bare-metal runtime or a provider for a particular microcontroller.

## Future Plans

Everything in this section is planned work. It is not part of the current implementation until the relevant code, tests, and documentation are merged.

### Phase 1 — cross-platform validation

- Run native builds, tests, and discovery on Linux and macOS.
- Confirm the supported Windows versions and representative hardware.
- Evaluate Raspberry Pi/Linux and additional targets separately.
- Decide which Android, iOS, and BSD targets are practical for this runtime and its host adapter.
- Add GitHub Actions workflows and a platform matrix after the native test strategy is defined.

### Phase 2 — richer hardware capability discovery

- Add GPU driver versions and more reliable device identity that remains privacy-conscious.
- Probe CUDA, ROCm, Metal, Vulkan, and OpenCL capabilities where the platform and drivers allow.
- Improve accelerator memory/topology reporting and capability scoring.
- Add battery, power, firmware, and additional sensor information where supported.
- Explore physical drive, controller, and storage topology without collecting unnecessary identifiers.

### Phase 3 — workflow runtime

- Define versioned workflow and DAG schemas.
- Add validation, deterministic state transitions, lifecycle management, and execution semantics.
- Design cancellation, scheduling, concurrency limits, and resource budgets.
- Add structured events, diagnostics, and recovery semantics.
- Keep the runtime headless and independent of the desktop editor.

### Phase 4 — local inference

- Design model metadata, loading, memory ownership, and lifecycle contracts.
- Implement in-process Rust inference backends rather than requiring an external Ollama/Python/container runtime.
- Add CPU and platform-specific acceleration backends behind stable interfaces.
- Support local, offline model operation and clear resource/error reporting.
- Add performance benchmarks only after correctness and resource behavior are defined.

### Phase 5 — sandboxing and package format

- Design a capability-restricted WASI Component Model execution boundary, potentially backed by Wasmtime.
- Specify the `.dea` package format, including bounded parsing, versioning, alignment, integrity, and compatibility rules.
- Add signature verification using standard algorithms and caller-controlled trust roots.
- Threat-model package parsing, model loading, component capabilities, and update paths.
- Do not treat a signature as proof that package contents are safe to execute.

### Phase 6 — developer, desktop, and embedded integrations

- Define stable client interfaces for Rust and potential TypeScript, Python, and Go SDKs.
- Integrate Daedalus Desktop through a documented local protocol or IPC boundary.
- Keep hosted marketplace services separate from local runtime execution and make their integration explicit.
- Add HAL/BSP/RTOS providers and target-specific runtime/transport profiles for embedded and edge devices.
- Validate each embedded target on actual hardware before claiming support.

## Agent and Project Documentation

The repository uses the same `agents-files/` organization as the Evergreen Estates project:

| File | Purpose |
|---|---|
| [AGENTS.md](./agents-files/AGENTS.md) | Required agent workflow, implementation constraints, and verification commands |
| [Agents_Context.md](./agents-files/Agents_Context.md) | Current project state, test evidence, and next actions |
| [ARCHITECTURE.md](./agents-files/ARCHITECTURE.md) | Single durable architecture authority |
| [CLAUDE.md](./agents-files/CLAUDE.md) | Claude Code imports for the agent documents |
| [SYSTEM_DISCOVERY.md](./docs/SYSTEM_DISCOVERY.md) | Detailed hardware-discovery API reference |

The context file is intentionally short-lived. Update it when new tests, platform validation, or implementation milestones are completed. Durable architecture decisions belong in `agents-files/ARCHITECTURE.md`.

## Licensing and Release Status

This repository is publicly visible on GitHub, but public visibility does not make it open source or grant redistribution rights. The checked-in `LICENSE` currently states All Rights Reserved and describes the software as proprietary and confidential. Apache License 2.0 and DCO contribution terms have not been applied.

Do not describe this repository as Apache-licensed or freely redistributable unless the owner deliberately changes the checked-in license, adds the corresponding contribution terms and notices, and reviews the repository's contents for publication. The README itself does not grant a license.

The separate Daedalus Desktop and Daedalus Marketplace products remain private.
