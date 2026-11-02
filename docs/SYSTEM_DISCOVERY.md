# System Discovery API

## Goal

System discovery reports the resources and platform capabilities that Daedalus can reliably observe. It is designed for model/workflow admission decisions, diagnostics, and client presentation.

The baseline implementation reports OS identity, architecture, CPU identity/topology, and physical memory. It does not collect process lists or persistent device identifiers.

## Public crate boundaries

### `daedalus-system-info`

This is the portable contract crate:

- `#![no_std]`
- No heap allocator required.
- Fixed-capacity `heapless::String` fields.
- Daedalus-owned normalized data structures.
- The `SystemInfoProvider` trait.
- Explicit `DiscoveryError`, `DiscoveryStatus`, and optional values.

A board-support package, RTOS integration, or application can implement `SystemInfoProvider` with target-specific readings.

### `daedalus-system-discovery`

This is the standard-library host adapter. It currently uses `sysinfo` for supported desktop and server operating systems.

The provider retains a single `sysinfo::System` instance. CPU identity/topology is enumerated at initialization; RAM is refreshed for each discovery request. It does not call `refresh_all` and does not enumerate processes, users, disks, networks, or temperature sensors.

Unsupported platforms are reported as `UNSUPPORTED_PLATFORM`. Data that is unavailable is represented as `null` in the JSON response.

## Runtime method

Method name: `hardware.discover`

Protocol version: `1`

Parameters: empty JSON object.

Example request:

```json
{
  "protocolVersion": 1,
  "requestId": "hardware-001",
  "method": "hardware.discover",
  "params": {}
}
```

Example response (illustrative values only):

```json
{
  "protocolVersion": 1,
  "requestId": "hardware-001",
  "ok": true,
  "result": {
    "schemaVersion": 1,
    "status": "complete",
    "platform": {
      "osFamily": "windows",
      "osName": "Windows",
      "osVersion": "Example Version",
      "kernelVersion": "Example Kernel",
      "architecture": "x86_64",
      "architectureName": "x86_64"
    },
    "processor": {
      "model": "Example CPU",
      "vendor": "Example Vendor",
      "logicalProcessors": 12,
      "physicalCores": 6
    },
    "memory": {
      "totalBytes": 17179869184,
      "availableBytes": 8589934592
    }
  }
}
```

### Field semantics

- `schemaVersion` versions the snapshot structure independently from the transport envelope.
- `status` is `complete` when baseline OS/architecture/CPU-count/memory data is present; otherwise it is `partial`.
- `osFamily` is a normalized family such as `windows`, `linux`, `macos`, `android`, `ios`, `freebsd`, `netbsd`, `bare-metal`, or `other`.
- `architecture` is a normalized architecture label; `architectureName` retains the provider's raw architecture string.
- CPU model/vendor and physical core count are optional because platforms may not expose them.
- Memory values are byte counts. `availableBytes` describes memory available for reuse, not merely completely unallocated memory.
- Missing fields are JSON `null`, not zero or fabricated labels.

## Direct CLI use

From the workspace root:

```bash
cargo run -p daedalus-cli -- hardware
cargo run -p daedalus-cli -- info
cargo run -p daedalus-cli -- ping
```

The CLI invokes the same runtime engine and host provider used by the daemon. It is intentionally a thin client, not a second hardware-discovery implementation.

## Embedded implementation

Bare-metal firmware should depend on `daedalus-system-info` and implement `SystemInfoProvider` using its HAL/BSP or RTOS APIs.

The host provider, JSON-over-stdio transport, runtime engine, and CLI use `std`. Compiling the contract crate for a Cortex-M target does not mean the full daemon or workflow runtime runs on that microcontroller.

The initial CI job cross-checks only the contract crate for `thumbv7em-none-eabihf`. Hardware-specific providers and runtime profiles require tests on actual boards.

## Planned extensions

GPU and accelerator discovery is deliberately separate from this baseline. A later adapter can report devices, vendor/model, device memory, driver/API availability, and supported compute backends. The public snapshot should continue to use Daedalus-owned types, and fields that cannot be read should remain optional.
