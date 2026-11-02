# Daedalus

**Daedalus is an open, local-first AI workflow runtime written in Rust.**

This repository is the public core of the Daedalus ecosystem. It is intended to provide a reusable, headless execution runtime that developers, researchers, enterprises, and government teams can build, inspect, test, and integrate without depending on the proprietary Daedalus desktop application or hosted marketplace.

The desktop product and marketplace are maintained separately and remain private.

## Repository boundaries

| Repository | Visibility | Responsibility |
|---|---|---|
| **[Daedalus](https://github.com/BryanReyes-dev/Daedalus)** | Public | Rust runtime, public protocol, CLI, system discovery, workflow execution, open package specifications, and developer SDKs |
| **Daedalus Desktop** | Private | Next.js/React interface, Electron shell, visual workflow editor, application experience, and proprietary client integrations |
| **[Daedalus Marketplace](https://github.com/BryanReyes-dev/Daedalus-Marketplace)** | Private | Storefront, creator services, artifact distribution, commercial entitlements, payments, and payouts |

The public runtime must remain useful on its own. It must not require the desktop app, a marketplace account, a remote licensing server, or a network connection for its core local execution functionality.

## Project goals

### 1. Headless, cross-platform runtime

The runtime is designed to execute independently of any UI framework. The initial supported desktop operating systems are:

- Windows
- Linux
- macOS

The same runtime contracts should be reusable by desktop, CLI, headless, and future embedded clients where the platform supports the required capabilities. Platform support will be documented and tested per target rather than assumed from successful compilation alone.

### 2. Hardware and system discovery

The runtime should expose a structured, privacy-conscious description of the host machine so higher-level applications can make informed decisions about model and workflow execution.

The initial discovery milestone targets:

- Operating system, version, and architecture.
- CPU model and logical processor count.
- Total and available system memory where the platform provides it.
- Explicit capability availability and unsupported/unknown values.

GPU discovery, accelerator capabilities, driver details, device memory, and detailed topology will be added through platform-specific adapters after the initial cross-platform baseline is tested.

System discovery should not silently collect serial numbers, persistent device identifiers, or unrelated personal data.

### 3. Portable workflow execution

The long-term runtime will provide:

- Portable workflow/DAG definitions and validation.
- Deterministic workflow state and lifecycle management.
- Scheduling, cancellation, and resource budgets.
- Native model loading and inference abstractions.
- Wasmtime/WASI execution for capability-restricted workflow components.
- A stable protocol for desktop and headless clients.

These capabilities are roadmap items until implemented, tested, and documented.

### 4. Open package and cryptographic standards

Public specifications should make package structure, integrity, and signature verification independently inspectable.

The runtime should support standard cryptographic verification against caller-provided trust roots. It must not depend on proprietary marketplace callbacks or secret verification algorithms to validate public package integrity.

Commercial package authorization and key release belong to the private desktop/marketplace integration. The public runtime should expose explicit, documented interfaces for consuming verified or authorized package content without implementing marketplace business logic.

A cryptographic library or build feature is not, by itself, evidence of FIPS 140-3 validation. Any FIPS claim must identify the validated cryptographic module, its validated operating environment, configuration, and deployment boundary.

## Intended architecture

The initial implementation is deliberately smaller than the complete roadmap.

```text
Desktop application / CLI / headless client
                    |
            Versioned public protocol
                    |
              runtime-ipc
                    |
              runtime-engine
              /     |      \
       system-info  workflows  execution
                    |
          platform-specific adapters
```

The runtime must not depend on Next.js, React, Electron, Tauri, or marketplace-specific services. Client applications own presentation and user interaction. The runtime owns host inspection and execution.

## Current implementation status

**The public repository is at the start of its extraction and foundation milestone.**

The initial Rust foundation currently exists on a development branch in the private combined repository and has not yet been migrated into this public repository. It provides a versioned JSON request/response protocol, a small command dispatcher, newline-delimited stdio transport, and a standalone daemon. At this point, the implemented runtime methods are limited to `runtime.ping` and `runtime.getInfo`.

The following are not yet implemented in this public repository:

- Cross-platform system and hardware discovery.
- Workflow DAG execution and scheduling.
- Native in-process inference.
- Wasmtime/WASI sandboxing.
- The `.dea` archive format and packer.
- Public-key signature verification.
- Runtime CLI.
- Platform CI and release artifacts.

Do not treat roadmap items as completed functionality.

## Near-term milestone: runtime foundation and system discovery

The first public milestone is complete when a developer can:

1. Build and test the Rust workspace independently of the desktop application.
2. Run the daemon headlessly on Windows, Linux, and macOS.
3. Send a versioned `hardware.discover` request over the public runtime interface.
4. Receive normalized OS, CPU, architecture, and memory information.
5. Get explicit unsupported or unavailable results rather than fabricated/default hardware values.
6. Run automated tests and cross-platform CI for the supported targets.
7. Read the implementation and reproduce the result using the public documentation.

GPU inventory and model-specific capability scoring are subsequent milestones.

## Development principles

- **Rust owns execution and system inspection.** UI frameworks display results; they do not implement runtime behavior.
- **Keep host-specific code behind adapters.** Public data models and protocol methods should not expose Windows, macOS, Linux, Electron, or Tauri implementation types.
- **Prefer explicit capability reporting.** Unknown, unavailable, unsupported, and detected are distinct states.
- **Minimize collection.** Discover resources required for execution, not personal or persistent device identity.
- **Test on real target operating systems.** Cross-compiling is useful, but it does not replace runtime tests on each OS.
- **Keep the core offline-capable.** Network-dependent commercial authorization must not be a prerequisite for ordinary local runtime operation.
- **Be precise about security claims.** Public source enables inspection; it does not automatically prove the system secure, certified, or compliant.

## Licensing and contributions

The intended public licensing model is Apache License 2.0 with Developer Certificate of Origin (DCO) sign-offs. The repository must include the corresponding license and contribution instructions before accepting external contributions. Until those files are present, do not assume that this README alone grants a license.

## Related private products

- **Daedalus Desktop:** proprietary desktop application that consumes the public runtime.
- **Daedalus Marketplace:** proprietary hosted storefront, creator, licensing, payment, and distribution services.

The public runtime is not the private desktop application and does not contain its UI, commercial storefront, marketplace database, payment system, or marketplace secrets.
