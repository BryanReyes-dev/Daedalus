# Daedalus

**Daedalus is an open, local-first AI workflow runtime written in Rust.**

This repository is the public core of the Daedalus ecosystem. It is designed to provide a reusable, headless runtime that desktop applications, command-line tools, services, and supported embedded clients can integrate without depending on a particular UI framework or hosted marketplace.

The proprietary desktop application and commercial marketplace are maintained separately.

## Repository boundaries

| Repository | Visibility | Responsibility |
|---|---|---|
| **[Daedalus](https://github.com/BryanReyes-dev/Daedalus)** | Intended public core | Rust runtime, public protocol, CLI, system discovery, workflow execution, and open specifications |
| **Daedalus Desktop** | Private | Next.js/React interface, Electron shell, visual workflow editor, application experience, and proprietary integrations |
| **[Daedalus Marketplace](https://github.com/BryanReyes-dev/Daedalus-Marketplace)** | Private | Storefront, creator services, artifact distribution, entitlements, payments, and payouts |

The public runtime must remain useful without the desktop app, a marketplace account, a remote licensing server, or a network connection for ordinary local operations.

## Current milestone: cross-platform system discovery

The first runtime milestone establishes a hardware/system information boundary that can be used by desktop operating systems and future embedded providers.

### Portable, allocation-free contract

`crates/daedalus-system-info` is a `no_std`, no-heap crate. It defines:

- Daedalus-owned platform, processor, memory, and snapshot data types.
- Fixed-capacity text values so the contract does not require a global allocator.
- Normalized OS-family and CPU-architecture identifiers.
- Explicit optional values for information a target cannot provide.
- The `SystemInfoProvider` trait that host adapters and embedded board-support packages can implement.

This is the portable contract. It does not access hardware by itself.

### Host operating-system provider

`crates/daedalus-system-discovery` is the `std` host adapter backed by `sysinfo`.

The first host implementation collects:

- OS family, name, version, and kernel version where available.
- CPU architecture, model, vendor, logical processor count, and physical core count where available.
- Total and currently available physical memory in bytes.

The provider retains its `sysinfo::System` instance, refreshes only the CPU metadata list at initialization and memory during discovery, and does not enumerate processes, users, disks, or network interfaces.

The `sysinfo` version currently used documents support for Windows, Linux, macOS, Android, iOS, FreeBSD, NetBSD, and Raspberry Pi-class systems. Fields may be unavailable depending on platform, permissions, virtualization, or hardware. Unsupported operations return explicit errors instead of fabricated values.

### Runtime protocol and CLI

The runtime exposes:

- `runtime.ping`
- `runtime.getInfo`
- `hardware.discover`

Run the CLI from the repository root:

```bash
cargo run -p daedalus-cli -- hardware
cargo run -p daedalus-cli -- info
cargo run -p daedalus-cli -- ping
```

The hardware command returns a versioned JSON response. A client application can request the same information through the runtime protocol/stdio daemon.

## Important embedded-platform boundary

The `daedalus-system-info` contract is `no_std` and uses fixed-capacity data structures; it can be implemented by bare-metal firmware, an RTOS port, or a board-support package.

The current `daedalus-system-discovery` host adapter, runtime engine, JSON protocol process, CLI, and daemon use `std`. They are not currently a complete bare-metal runtime and are not claimed to run on every microcontroller.

The embedded CI job checks that the portable contract compiles for `thumbv7em-none-eabihf`. A real embedded implementation still needs a target-specific provider and hardware testing.

## Intended architecture

```text
Private desktop / CLI / headless client / future embedded client
                           |
                   versioned protocol
                           |
                      runtime-ipc
                           |
                     runtime-engine
                       /        \
             system-discovery   other runtime capabilities
                    |
             system-info contract
                    |
       OS adapter / RTOS / board support package
```

The runtime must not depend on Next.js, React, Electron, Tauri, or marketplace-specific services. Client applications own presentation and user interaction. The runtime owns system inspection and execution.

## Current implementation status

This feature branch adds the following initial implementation:

- A no-std, allocation-free system information contract.
- A `sysinfo` host provider.
- The `hardware.discover` runtime method.
- A `daedalus` CLI with `hardware`, `info`, and `ping` commands.
- Unit tests for portable type normalization, host discovery, runtime dispatch, and protocol handling.
- CI for native Windows, Linux, and macOS runs plus a no-std embedded-target compile check.

These changes have been committed to the feature branch, but they have **not yet been built or tested locally by the maintainer**. The lockfile must be regenerated and committed after pulling the feature branch. Do not treat this branch as a verified release until its local and CI checks pass.

GPU inventory, accelerator enumeration, driver versions, VRAM, workflow scheduling, in-process inference, Wasmtime/WASI sandboxing, the `.dea` archive format, and public-key signature verification are subsequent milestones.

## Privacy and discovery principles

- Collect only system information required for capability planning.
- Do not collect serial numbers, persistent device identifiers, MAC addresses, process lists, or unrelated personal data as part of baseline discovery.
- Treat unsupported or unavailable fields as absent rather than using fake defaults.
- Keep OS-specific calls behind providers; public protocol types must not expose third-party library types.
- Test on native target operating systems. Cross-compilation alone does not prove correct runtime discovery.
- Keep the portable contract independent of heap allocation, OS APIs, and UI frameworks.

## Development

Requirements: stable Rust and Cargo. The host adapter currently follows the `sysinfo` crate's Rust 1.95 minimum; embedded contract consumers may choose a separate toolchain target.

Useful commands:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo check -p daedalus-system-info --target thumbv7em-none-eabihf
```

After the new dependencies are resolved, commit the updated `Cargo.lock` so the workspace has a reproducible dependency graph.

## Licensing and contributions

The intended public licensing model is Apache License 2.0 with Developer Certificate of Origin (DCO) sign-offs. The repository still needs its actual Apache 2.0 license text and DCO contribution instructions installed and reviewed before it is made public or accepts external contributions. The README alone does not grant a license.

## Related private products

- **Daedalus Desktop:** proprietary desktop application that consumes the public runtime.
- **Daedalus Marketplace:** proprietary hosted storefront, creator, licensing, payment, and distribution services.

The public runtime does not contain the private desktop UI, commercial storefront, marketplace database, payment system, or marketplace signing secrets.
