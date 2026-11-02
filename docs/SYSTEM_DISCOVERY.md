# System Discovery API

> This is the focused hardware-discovery API reference. The durable architecture authority is [agents-files/ARCHITECTURE.md](../agents-files/ARCHITECTURE.md); agent workflow rules are in [agents-files/AGENTS.md](../agents-files/AGENTS.md); current evidence and next work are in [agents-files/Agents_Context.md](../agents-files/Agents_Context.md); user-facing setup and usage are in [README.md](../README.md).

## Purpose

System discovery reports platform identity and hardware resources that Daedalus can observe through its current host provider. It supports diagnostics and future model/workflow capability planning.

Discovery is best-effort. Operating systems expose different APIs and permission levels, so missing readings remain optional rather than being replaced with fabricated values.

## Crate boundaries

### daedalus-system-info

The portable contract crate provides no_std types and does not require a heap allocator. It uses fixed-capacity heapless strings and vectors, Daedalus-owned normalized types, a SystemInfoProvider trait, explicit discovery status and optional readings.

The crate contains no operating-system calls and does not inspect hardware itself.

### daedalus-system-discovery

The standard-library host adapter uses sysinfo 0.40.1. It retains its System, Disks, Components and Gpus provider instances where the current operating system supports them.

It refreshes CPU frequency, RAM/swap, storage, GPU and thermal values on demand. It reads product and motherboard model metadata when available. It does not request process/user lists, host names, network interfaces, MAC addresses, disk mount paths, serial numbers, UUIDs, asset tags or PCI addresses.

## Host platform coverage

The upstream sysinfo 0.40.1 project documents host OS support for Windows, Linux, macOS, Android, iOS, FreeBSD, NetBSD and Raspberry Pi-class Linux systems.

Categories have narrower support:

- GPU inventory: Windows, Linux and macOS.
- Product and motherboard metadata: not implemented on NetBSD.
- Thermal component inventory: not implemented on NetBSD; it may be unavailable in VMs, containers, WSL, sandboxed applications or systems without exposed sensors.
- Newer Android versions may restrict CPU details for ordinary applications.
- GPU usage and memory fields depend on driver support. macOS uses shared GPU/system memory, so dedicated VRAM values are not reported.

A supported host OS does not guarantee every category or every hardware revision will be detected. Individual native validation is still required.

## Protocol method

- Method: hardware.discover
- Protocol version: 1
- Parameters: empty JSON object.

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
      "physicalCores": 6,
      "frequencyMhz": 3600
    },
    "memory": {
      "totalBytes": 17179869184,
      "availableBytes": 8589934592,
      "usedBytes": 8589934592,
      "swapTotalBytes": 0,
      "swapAvailableBytes": 0,
      "swapUsedBytes": 0
    },
    "product": {
      "vendor": "Example Manufacturer",
      "name": "Example System",
      "family": "Example Product Family",
      "version": "Revision 1"
    },
    "motherboard": {
      "vendor": "Example Board Vendor",
      "model": "Example Board",
      "version": "Revision 1"
    },
    "storageDevices": [
      {
        "name": "Disk 0",
        "kind": "ssd",
        "fileSystem": "NTFS",
        "totalBytes": 1000204886016,
        "availableBytes": 700000000000,
        "removable": false,
        "readOnly": false
      }
    ],
    "gpus": [
      {
        "vendor": "Example GPU Vendor",
        "model": "Example GPU",
        "totalMemoryBytes": 8589934592,
        "usedMemoryBytes": 1073741824,
        "usagePercent": 18.5
      }
    ],
    "thermalSensors": [
      {
        "label": "CPU",
        "temperatureCelsius": 42.0,
        "maximumCelsius": 66.0,
        "criticalCelsius": 95.0
      }
    ]
  }
}
```

## Snapshot field semantics

| Field | Meaning |
|---|---|
| schemaVersion | Snapshot structure version, independent of protocol version |
| status | complete when baseline OS label, architecture label, logical CPU count and total RAM exist; otherwise partial |
| platform | Normalized OS family, OS labels, kernel version and architecture |
| processor.frequencyMhz | Reported CPU frequency in MHz; not necessarily a sustained or all-core clock |
| memory.*Bytes | RAM and swap byte counts |
| product | Manufacturer/system model metadata; null when no supported model fields are available |
| motherboard | Board vendor/model/version; null when unsupported or not exposed |
| storageDevices | Disk/volume inventory when the storage API is available; `null` when it cannot be queried; mount paths are excluded |
| gpus | Array when GPU enumeration is available; null when the provider cannot query it |
| thermalSensors | Sensor array when the API is available; null when sensor enumeration cannot run |

Missing scalar readings serialize as JSON null. For `storageDevices`, `gpus`, and `thermalSensors`, an array of `[]` means the query ran but returned no entries; `null` means that category could not be queried. The portable contract limits a snapshot to 16 storage entries, 8 GPUs and 32 thermal sensors.

## Privacy boundary

The default provider intentionally omits motherboard/product serial numbers, UUIDs, asset tags, persistent device IDs, PCI bus addresses, network interface names, IP/MAC addresses, process data, user accounts, host names and disk mount points.

The information is still machine-specific. Applications should obtain consent before transmitting snapshots off-device.

## CLI use

Run from the workspace root:

```bash
cargo run -p daedalus-cli -- hardware
cargo run -p daedalus-cli -- info
cargo run -p daedalus-cli -- ping
cargo run -p daedalus-cli -- --help
cargo run -p daedalus-cli -- --version
```

The CLI invokes RuntimeEngine in-process and emits a pretty-printed JSON response. It shares the same default host provider as the daemon.

## Daemon protocol

```bash
printf '%s\n' '{"protocolVersion":1,"requestId":"hardware-001","method":"hardware.discover","params":{}}' | cargo run -p daedalus-runtime
```

The daemon processes newline-delimited JSON. One input line produces one response line. Malformed input returns INVALID_REQUEST and the daemon continues reading subsequent lines. Stdout is reserved for responses; diagnostics use stderr.

## Embedded provider implementation

Embedded firmware can depend on daedalus-system-info and implement SystemInfoProvider using its HAL, BSP or RTOS APIs.

The host provider, runtime engine, JSON protocol, CLI and daemon are std components. This milestone does not provide a complete bare-metal runtime or a provider for any particular microcontroller.

## Test guidance

Run tests from the workspace root:

```bash
cargo test -p daedalus-system-info
cargo test -p daedalus-system-discovery -- --test-threads=1
cargo test -p daedalus-runtime-engine
cargo test --workspace -- --test-threads=1
```

System inspection tests are best run serially.

## Project validation status

As of 2026-11-02, the project author reported that the workspace formatting check, workspace compile check, full workspace test suite, and CLI smoke tests passed on Windows 11. This is local user-reported evidence; it is not a GitHub Actions result.

Native execution and hardware validation on Linux, macOS, Android/iOS, BSD, and Raspberry Pi devices remain pending. Upstream `sysinfo` platform coverage and cross-compilation do not establish that this repository's provider works correctly on those targets. Per-platform native tests and GitHub Actions are intentionally deferred to a later implementation task.

## Planned extensions

- Native platform test suites.
- Bare-metal providers and embedded runtime profiles.
- GPU driver and compute API probing for CUDA, ROCm, Metal, Vulkan and OpenCL.
- Accelerator identification and workload scheduling.
- Battery, power, firmware and richer sensor inventory.
