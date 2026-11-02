# Daedalus Runtime Architecture

## Purpose

Daedalus is a headless Rust runtime. It must not depend on a desktop UI framework, web frontend, marketplace service or commercial account system for ordinary local operation.

The public runtime is intended to be consumed by independent desktop, CLI, headless and future embedded clients.

## Workspace

| Path | Responsibility | Runtime model |
|---|---|---|
| crates/runtime-protocol | Versioned request/response data structures | std today |
| crates/runtime-engine | Command dispatch and provider orchestration | std |
| crates/runtime-ipc | Newline-delimited JSON over stdin/stdout | std |
| crates/daedalus-system-info | Portable system data model and provider trait | no_std, no heap |
| crates/daedalus-system-discovery | Host OS provider backed by sysinfo | std |
| apps/runtime-daemon | Standalone JSON-over-stdio daemon | std |
| apps/daedalus-cli | In-process command-line inspector | std |

Implemented protocol methods are runtime.ping, runtime.getInfo and hardware.discover.

## Dependency direction

```text
runtime-daemon -- runtime-ipc -- runtime-engine
                                     |
                             system-discovery
                                     |
                              system-info
daedalus-cli ------------------------/
```

The runtime engine depends on the provider interface, not directly on operating-system calls. The default host provider is SysinfoProvider. Tests and alternate targets can inject another SystemInfoProvider.

## Portable contract

daedalus-system-info is no_std and uses fixed-capacity heapless strings and vectors. It contains normalized fixed-width types, optional readings and the SystemInfoProvider trait. It performs no hardware access and requires no allocator.

The contract includes platform, CPU, RAM/swap, product and motherboard identity, storage, GPU and thermal structures. Provider implementations can populate only the fields their target can actually read.

## Host discovery adapter

daedalus-system-discovery is a std adapter backed by sysinfo 0.40.x.

The host provider:
1. Initializes and retains one System instance.
2. Refreshes CPU frequency and CPU topology data.
3. Refreshes RAM and swap information.
4. Enumerates storage entries and capacity data without returning mount paths.
5. Queries GPU inventory where that API is supported.
6. Queries exposed thermal components.
7. Reads product and motherboard model metadata without serial numbers or asset tags.
8. Converts all readings into Daedalus-owned data types.

It does not request process or user lists, host names, networks, MAC addresses, persistent device identifiers, PCI addresses or disk mount points.

The upstream sysinfo host support list includes Windows, Linux, macOS, Android, iOS, FreeBSD, NetBSD and Raspberry Pi-class Linux. GPU inventory has narrower support: Windows, Linux and macOS. Product/motherboard and thermal APIs also have platform gaps. VMs, containers, sandboxed applications and permissions can limit the values exposed by the host.

## Runtime hardware method

hardware.discover accepts an empty params object in protocol version 1. The result includes schemaVersion, status, platform, processor, memory, product, motherboard, storageDevices, gpus and thermalSensors.

status is complete if baseline OS label, architecture label, logical processor count and total memory are present; otherwise it is partial. Missing scalar fields are JSON null. Optional GPU and thermal arrays use null when the category is unavailable and [] when the query ran but returned no entries.

The snapshot schema has its own schemaVersion. The JSON request/response envelope remains protocol version 1.

## CLI and daemon

The daedalus CLI calls RuntimeEngine in-process. It supports hardware, info, ping, help and version switches. Runtime commands output pretty-printed JSON.

The daedalus-runtime daemon uses runtime-ipc to process newline-delimited JSON on stdin and write one response line per request to stdout. Diagnostics go to stderr.

## Embedded scope

The portable system-info crate can be compiled for no_std targets. The host adapter, runtime engine, JSON protocol, CLI and daemon are std components. This milestone does not claim full bare-metal runtime support.

Future embedded work requires a target-specific provider using its HAL/BSP/RTOS, an appropriate transport or in-process profile, and tests on the actual board.

## Validation status

Unit tests are maintained within the portable contract, host adapter, protocol, runtime engine and transport crates. Native testing on each intended OS and device class is deferred. GitHub Actions and per-platform automation are also deferred. Cross-compilation alone does not establish working device discovery.

## Security and privacy

- Keep ordinary runtime operation local-first and offline-capable.
- Reserve stdout for protocol messages; send diagnostics to stderr.
- Bound text and device collection sizes.
- Do not collect serial numbers, product UUIDs, asset tags, persistent hardware IDs, MAC addresses, process lists, user accounts or disk mount paths during normal discovery.
- Treat caller-provided package bytes and workflow definitions as untrusted input.
- Do not claim FIPS validation without an identified validated module and deployment boundary.

## Not implemented

Workflow execution, inference, Wasmtime/WASI sandboxing, package parsing, cryptographic package verification, .dea archives, GPU driver/compute API probing and accelerator scheduling remain future capabilities.
