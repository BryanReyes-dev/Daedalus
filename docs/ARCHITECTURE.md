# Daedalus Runtime Architecture

## Purpose

Daedalus is a headless Rust runtime. It must not depend on a desktop UI framework, web frontend, marketplace service, or commercial account system for ordinary local operation.

The public runtime is intended to be consumed by independent desktop, CLI, headless, and future embedded clients.

## Current workspace

| Path | Responsibility | Runtime model |
|---|---|---|
| `crates/runtime-protocol` | Versioned request/response data structures | `std` today |
| `crates/runtime-engine` | Host-side command dispatch and capability orchestration | `std` |
| `crates/runtime-ipc` | Newline-delimited JSON over stdin/stdout | `std` |
| `crates/daedalus-system-info` | Portable system data model and provider trait | `no_std`, no heap |
| `crates/daedalus-system-discovery` | Host OS provider backed by sysinfo | `std` |
| `apps/runtime-daemon` | Standalone JSON-over-stdio daemon | `std` |
| `apps/daedalus-cli` | Human-invoked runtime inspection commands | `std` |

The current protocol methods are:
- `runtime.ping`
- `runtime.getInfo`
- `hardware.discover`

Workflow execution, inference, Wasmtime/WASI sandboxing, package parsing, cryptographic verification, and GPU inventory are not implemented by this milestone.

## Dependency direction

```text
runtime-daemon / daedalus-cli
             |
        runtime-ipc (daemon only)
             |
        runtime-engine
          /       \
system-discovery  runtime-protocol
       |
system-info contract
```

The runtime engine depends on the provider interface, not directly on operating-system calls. The default desktop/server provider is `SysinfoProvider`; tests and alternate targets can inject another `SystemInfoProvider`.

## Portable system information contract

`daedalus-system-info` is the portability boundary:

- `#![no_std]`.
- No dynamic memory allocation.
- Fixed-capacity `heapless::String` values for bounded text.
- Numeric resource values use fixed-width integer types.
- Unknown or unavailable values are represented by `Option`.
- The provider trait reports a normalized `SystemSnapshot`.

The crate performs no hardware access. Embedded firmware, RTOS integrations, and board-support packages implement `SystemInfoProvider` using the APIs available on their own target.

This separates a portable data contract from hardware-specific acquisition. A microcontroller implementation can return only the measurements it can actually obtain.

## Host system discovery

`daedalus-system-discovery` is a `std` adapter using `sysinfo` 0.40.x.

The provider:
1. Initializes one `sysinfo::System` instance lazily.
2. Enumerates CPU identity/topology once.
3. Refreshes RAM information when discovery is requested.
4. Normalizes the result into Daedalus-owned structures.
5. Reports unavailable values as absent and unsupported platforms as explicit errors.

It must not enumerate processes, users, network interfaces, or disks as part of baseline hardware discovery.

The initial host adapter is intended for Windows, Linux, and macOS and can also use other platforms supported by the selected `sysinfo` release. Actual runtime support must be established by native CI and device tests. Virtual machines and containers may report guest-visible rather than physical-host resource information.

## Runtime API

The `hardware.discover` request takes an empty parameter object in protocol version 1 and returns:

- `schemaVersion`
- `status` (`complete` or `partial`)
- `platform`: OS family/name/version, kernel version, and normalized/raw architecture
- `processor`: model, vendor, logical processor count, physical core count
- `memory`: total and available bytes

Fields that cannot be read are represented as JSON `null`; the runtime must not substitute invented values. The response is generated from Daedalus-owned types rather than serializing `sysinfo` types directly.

The addition of this method does not alter the version 1 request/response envelope.

## Embedded scope and limitations

The portable `daedalus-system-info` crate is designed to compile without `std` or a heap allocator. The current host provider, runtime engine, JSON protocol, CLI, and daemon are still `std` components.

Consequently, this milestone does **not** claim that the complete daemon/runtime can run on bare-metal microcontrollers. Future embedded work requires:
- A target-specific `SystemInfoProvider` implementation using its HAL/BSP/RTOS.
- A suitable transport or in-process integration for that target.
- An execution/runtime profile appropriate to its memory, timing, and peripheral constraints.
- Tests on the actual development board.

The initial CI target checks only that the portable data/trait crate compiles for `thumbv7em-none-eabihf`; this is a compile check, not proof of a complete embedded runtime.

## Cross-platform validation

Native CI should run formatting, workspace compilation, and tests on:
- Windows
- Linux
- macOS

The no-std contract should additionally be checked against at least one bare-metal target. A successful cross-compilation does not replace runtime testing on a real target device.

## Security and privacy

- Keep runtime operation local-first and offline-capable.
- Write protocol responses to stdout only; diagnostics go to stderr.
- Bound request sizes and validate all protocol requests before dispatch.
- Do not collect serial numbers, persistent hardware identifiers, MAC addresses, process lists, or unrelated personal data for ordinary discovery.
- Treat caller-provided package bytes and workflow definitions as untrusted input.
- Do not claim FIPS validation without an identified validated module and deployment boundary.

## Future architecture (not implemented)

Potential future crates and capabilities include:
- `workflow-core`
- `workflow-engine`
- `runtime-inference`
- `runtime-sandbox`
- `runtime-crypto`
- `dea-format`
- A fuller headless CLI and embedded runtime profile
- Platform-specific GPU and accelerator adapters

Add crates only when concrete functionality and independent testing needs justify them.
