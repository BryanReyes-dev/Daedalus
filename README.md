# Daedalus

**Daedalus is a local-first, headless AI workflow runtime written in Rust.**

This repository contains the reusable Rust runtime core, its versioned JSON protocol, a command-line inspector, and host system-discovery capabilities. It is designed to work independently of the desktop interface and hosted marketplace. Ordinary local runtime operations do not require a network connection.

The proprietary desktop application and marketplace are maintained separately.

## Repository boundaries

| Repository | Responsibility |
|---|---|
| [Daedalus](https://github.com/BryanReyes-dev/Daedalus) | Rust runtime, protocol, CLI, system discovery, and open specifications |
| Daedalus Desktop | Private Next.js/React interface, Electron shell, visual workflow editor, and proprietary application integrations |
| [Daedalus Marketplace](https://github.com/BryanReyes-dev/Daedalus-Marketplace) | Private storefront, creator services, artifact distribution, entitlements, payments, and payouts |

The runtime must remain useful without the desktop app, a marketplace account, a remote licensing server, or a network connection for ordinary local operations.

## Current implementation

The current system-discovery milestone provides:

- A portable no_std, no-heap system-information contract.
- Host operating-system and CPU-architecture normalization.
- CPU model, vendor, logical processor count, physical core count, and reported frequency.
- RAM and swap capacity, available and used byte counts.
- Product and motherboard manufacturer/model/version metadata where exposed.
- Storage inventory with device type, filesystem, capacity, available space, removable and read-only state.
- GPU vendor/model, VRAM and utilization where the operating system and driver expose them.
- Thermal sensors and current/maximum/critical temperature readings where exposed.
- A versioned JSON request/response protocol.
- A daedalus CLI and a separate daedalus-runtime newline-delimited JSON daemon.
- Unit tests in the portable contract, host provider, protocol, runtime engine and transport crates.

The host adapter does not enumerate processes, user accounts, host names, network interfaces, IP addresses, MAC addresses, disk mount paths, hardware serial numbers, product UUIDs, asset tags, or PCI bus addresses.

## Platform coverage and limits

The host adapter uses [sysinfo 0.40.1](https://docs.rs/sysinfo/0.40.1/sysinfo/). Its documented host OS support includes Windows, Linux, macOS, Android, iOS, FreeBSD, NetBSD, and Raspberry Pi-class Linux systems.

Not every information category is available on every supported OS:

- GPU inventory is supported by the upstream API on Windows, Linux and macOS.
- Product and motherboard metadata are not implemented on NetBSD.
- Thermal component inventory is not implemented on NetBSD and may be absent in containers, WSL, virtual machines, sandboxed applications, or systems without exposed sensors.
- Newer Android versions restrict ordinary applications from reading some CPU details.
- GPU utilization and memory availability depend on drivers. macOS uses shared GPU/system memory, so dedicated VRAM fields may be null.
- Storage inventory represents the volumes exposed by the OS and does not promise physical-drive topology or controller/RAID detail.

A supported host OS does not guarantee every category or every hardware revision will be detected. Virtual machines and containers may report guest-visible rather than physical-host resources. Raspberry Pi discovery uses its Linux host environment.

### Embedded boundary

The portable daedalus-system-info crate is no_std and does not require a heap allocator. It defines the SystemInfoProvider trait that firmware, RTOS integrations, and board-support packages can implement using their own HAL/BSP.

The current host adapter, runtime engine, JSON process, CLI and daemon use std. They are not a complete bare-metal runtime. A microcontroller requires a target-specific provider and an appropriate runtime/transport profile.

## Quick start

### Requirements

- Rust and Cargo.
- Rust 1.95 or newer for the current sysinfo dependency.
- A host operating system supported by the host discovery backend for live host information.

Check your installation:

```bash
rustc --version
cargo --version
```

Clone the repository and enter its root:

```bash
git clone https://github.com/BryanReyes-dev/Daedalus.git
cd Daedalus
```

The system-discovery implementation is currently on feat/system-discovery and has not yet been merged into main:

```bash
git switch feat/system-discovery
```

Build the workspace:

```bash
cargo build --workspace
```

Run the CLI:

```bash
cargo run -p daedalus-cli -- hardware
cargo run -p daedalus-cli -- info
cargo run -p daedalus-cli -- ping
```

## Complete CLI reference

The command-line executable is named daedalus. During development, invoke it through Cargo from the workspace root:

```bash
cargo run -p daedalus-cli -- <COMMAND>
```

After a release build, the executable is located at target/release/daedalus.exe on Windows and target/release/daedalus on Linux/macOS. You can run the compiled binary directly from that path.

### daedalus hardware

Discovers the local platform and host hardware and writes a pretty-printed JSON runtime response:

```bash
cargo run -p daedalus-cli -- hardware
```

It does not upload, persist, or transmit the snapshot.

The CLI prints a standard runtime response envelope. For a successful hardware request, the hardware snapshot is inside the envelope's `result` object. Its fields are:

| Field in `result` | Description |
|---|---|
| `schemaVersion` | Hardware-snapshot schema version |
| `status` | `complete` when baseline OS, architecture, logical CPU count and total RAM fields exist; otherwise `partial` |
| `platform` | OS family/name/version, kernel version, normalized and raw CPU architecture |
| `processor` | CPU model/vendor, logical processors, physical cores and reported frequency |
| `memory` | RAM and swap totals, available and used bytes |
| `product` | Manufacturer/system model/family/version when available |
| `motherboard` | Board manufacturer/model/version when available |
| `storageDevices` | Storage kind, filesystem, capacity/free bytes, removable/read-only flags |
| `gpus` | GPU inventory, or `null` when the host provider cannot query it |
| `thermalSensors` | Temperature sensor readings, or `null` when sensor enumeration is unavailable |

Memory and storage sizes are byte counts. CPU frequency is MHz. GPU utilization is a percentage. Temperatures are degrees Celsius.

Unavailable scalar values are JSON null. Optional device arrays distinguish null (category could not be queried) from an empty array (query succeeded but returned no devices). The portable contract retains up to 16 storage entries, 8 GPUs and 32 thermal sensors per snapshot.

The complete status refers only to baseline platform/CPU/RAM fields. It does not guarantee that motherboard, GPU, storage, or sensor details are all populated.

### daedalus info

Prints the runtime name, version, status, protocol version and implemented methods:

```bash
cargo run -p daedalus-cli -- info
```

Implemented methods:

- runtime.ping
- runtime.getInfo
- hardware.discover

### daedalus ping

Checks that the in-process runtime engine responds:

```bash
cargo run -p daedalus-cli -- ping
```

This checks the engine inside the CLI process. It does not start or contact the separate daemon.

### Help and version

```bash
cargo run -p daedalus-cli -- help
cargo run -p daedalus-cli -- --help
cargo run -p daedalus-cli -- --version
```

The CLI accepts one command at a time and currently has no command-specific flags. Results go to stdout as JSON. Usage errors go to stderr.

### Exit codes

| Code | Meaning |
|---:|---|
| 0 | Command succeeded |
| 1 | Runtime/provider or response-serialization failure |
| 2 | Unknown command or unexpected extra argument |

## Runtime daemon and JSON stdio protocol

The workspace also builds a separate process named daedalus-runtime. Unlike the CLI, the daemon reads newline-delimited JSON requests from stdin and writes one JSON response per line to stdout.

Start it:

```bash
cargo run -p daedalus-runtime
```

From Git Bash, send a ping request:

```bash
printf '%s\n' '{"protocolVersion":1,"requestId":"ping-001","method":"runtime.ping","params":{}}' | cargo run -p daedalus-runtime
```

Send a hardware-discovery request:

```bash
printf '%s\n' '{"protocolVersion":1,"requestId":"hardware-001","method":"hardware.discover","params":{}}' | cargo run -p daedalus-runtime
```

The daemon accepts multiple input lines in one process. Every non-empty line produces one response line. Malformed JSON produces an INVALID_REQUEST response and does not stop subsequent requests. Stdout is reserved for protocol messages; diagnostics are written to stderr.

### Request envelope

```json
{
  "protocolVersion": 1,
  "requestId": "hardware-001",
  "method": "hardware.discover",
  "params": {}
}
```

- protocolVersion must match the runtime's current protocol version.
- requestId is copied into the response for correlation.
- method selects the operation.
- params is an optional JSON object. Current methods do not require parameters.

### Successful response

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

### Failure response

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

The unused result or error property is omitted. Hardware snapshot schemaVersion and transport protocolVersion are separate version numbers.

### Runtime error codes

| Code | Meaning |
|---|---|
| INVALID_REQUEST | An input line could not be parsed |
| PROTOCOL_VERSION_UNSUPPORTED | Client and runtime protocol versions differ |
| METHOD_NOT_FOUND | The requested method is not implemented |
| UNSUPPORTED_PLATFORM | No host provider is available for this target |
| PROVIDER_INITIALIZATION_FAILED | A host provider could not initialize |
| SYSTEM_DATA_UNAVAILABLE | A provider could not read required data |

## Workspace layout

```text
apps/
  daedalus-cli/       # Human-invoked command-line inspector
  runtime-daemon/     # JSON-over-stdio runtime process
crates/
  daedalus-system-info/       # no_std portable data contract and provider trait
  daedalus-system-discovery/  # std host adapter using sysinfo
  runtime-engine/             # Method dispatch and provider orchestration
  runtime-ipc/                # Newline-delimited JSON transport
  runtime-protocol/           # Versioned request/response structures
docs/
  ARCHITECTURE.md
  SYSTEM_DISCOVERY.md
```

### Crate responsibilities

- daedalus-system-info owns normalized, bounded Rust types. It contains no OS calls and does not require std or a heap allocator.
- daedalus-system-discovery accesses host OS APIs through sysinfo and converts readings into the portable contract.
- runtime-engine dispatches protocol methods and invokes injected providers.
- runtime-protocol defines the request/response envelope.
- runtime-ipc implements the daemon's line-oriented stdin/stdout transport.
- daedalus-cli invokes RuntimeEngine directly in-process.
- runtime-daemon exposes the same engine through stdio.

## Development and test guide

Run all commands from the workspace root.

### Format and compile

```bash
cargo fmt --all
cargo fmt --all -- --check
cargo check --workspace
cargo build --workspace
cargo build --workspace --release
```

### Run the full test suite

```bash
cargo test --workspace -- --test-threads=1
```

Serial execution is recommended for tests that inspect host system information; this reduces interference between concurrent OS queries.

### Run one crate's tests

```bash
cargo test -p daedalus-system-info
cargo test -p daedalus-system-discovery -- --test-threads=1
cargo test -p daedalus-runtime-protocol
cargo test -p daedalus-runtime-engine
cargo test -p daedalus-runtime-ipc
```

### Check the portable no_std contract for a bare-metal target

```bash
rustup target add thumbv7em-none-eabihf
cargo check -p daedalus-system-info --target thumbv7em-none-eabihf
```

This compiles only the portable data contract. It does not make the std host adapter, runtime engine, daemon or CLI run on a microcontroller.

### Smoke-test the CLI and daemon

```bash
cargo run -p daedalus-cli -- hardware
cargo run -p daedalus-cli -- info
cargo run -p daedalus-cli -- ping
printf '%s\n' '{"protocolVersion":1,"requestId":"ping-001","method":"runtime.ping","params":{}}' | cargo run -p daedalus-runtime
```

### Cargo.lock policy

Cargo.lock is committed so the workspace uses a reproducible dependency resolution. After changing dependencies or enabling dependency features, regenerate and commit the lockfile:

```bash
cargo check --workspace
git add Cargo.toml Cargo.lock crates/daedalus-system-discovery/Cargo.toml
git commit -m "chore: update Rust dependency lockfile"
```

## Platform validation status

Unit tests are included in the workspace, but native validation on each intended operating system and hardware class is a separate milestone. Run discovery on actual Windows, Linux, macOS, Android/iOS where applicable, BSD and Raspberry Pi targets before describing those targets as verified.

Per-platform native test runs and GitHub Actions workflows are intentionally deferred. A compile check or cross-compilation is not proof of runtime discovery on actual hardware.

## Privacy and safe discovery

System discovery is intended for local capability inspection, not telemetry collection.

The default provider does not return process lists, executable paths, command-line arguments, user accounts, host names, network interfaces, IP addresses, MAC addresses, disk mount paths, serial numbers, product UUIDs, asset tags, or PCI addresses.

Returned system information is still machine-specific. A client must obtain the machine owner's consent before transmitting a snapshot outside the device.

## Current limitations and next milestones

Implemented in this milestone:

- Platform, architecture, CPU, RAM and swap discovery.
- Product and motherboard model metadata where exposed.
- Storage, GPU and thermal inventory where host APIs make it available.
- Versioned JSON protocol, CLI and stdio daemon.

Not implemented yet:

- Per-platform native test runs and GitHub Actions.
- Bare-metal board providers and a complete embedded runtime profile.
- GPU driver versions, CUDA/ROCm/Metal/Vulkan/OpenCL capability probing, accelerator scheduling and compute benchmarks.
- Battery, power, firmware and richer sensor inventory.
- Workflow execution, inference engines, model loading, Wasmtime/WASI sandboxing, package parsing, the .dea archive format, and cryptographic package-signature verification.

## Licensing and contributions

The intended licensing model is Apache License 2.0 with Developer Certificate of Origin (DCO) sign-offs. The repository still needs its actual Apache 2.0 license text and DCO contribution instructions installed and reviewed before it is made public or accepts external contributions. This README does not itself grant a license.

## Related private products

- Daedalus Desktop: proprietary desktop application that consumes the public runtime.
- Daedalus Marketplace: proprietary hosted storefront, creator, licensing, payment and distribution services.

The public runtime does not contain the private desktop UI, storefront, marketplace database, payment system or marketplace signing secrets.
