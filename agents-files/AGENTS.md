# Daedalus — Agent Instructions

Daedalus is a local-first Rust runtime project. The current implementation is a foundation consisting of a portable system-information contract, a host discovery adapter, a versioned JSON protocol, a runtime dispatcher, a stdio transport, a CLI, and a daemon. Workflow execution, inference, package loading, sandboxing, and marketplace integration are roadmap items unless implemented and verified in source.

## Required reading order

1. Read [Agents_Context.md](./Agents_Context.md) for the current implementation state, verified evidence, and immediate priorities.
2. Read [ARCHITECTURE.md](./ARCHITECTURE.md) for the durable architecture, crate boundaries, data contracts, and security constraints.
3. Read the human-facing [README](../README.md) for setup, CLI usage, test commands, platform limits, and roadmap.
4. Read [SYSTEM_DISCOVERY.md](../docs/SYSTEM_DISCOVERY.md) before changing the system-discovery contract or host adapter.
5. Inspect the relevant Cargo manifests, source files, and tests before editing.

## Safe repository workflow

- Check the current branch, working-tree status, latest commit, and remote state before proposing or applying changes.
- Work on a focused feature or documentation branch. Do not commit directly to `main` unless the user explicitly asks.
- Preserve unrelated local modifications. Never use `git restore`, reset, clean, force-push, or history rewriting as a shortcut without checking what would be discarded and obtaining clear authorization where needed.
- Keep changes scoped to the requested task. Explain protocol, data-model, security, licensing, or compatibility changes.
- Do not claim a build, test, platform, CI run, release, deployment, or hardware result unless it was actually observed. Mark user-reported test evidence as user-reported.
- Do not silently add GitHub Actions or other hosted CI. Cross-platform workflows are a planned follow-up and require an explicit task.
- Do not change repository visibility or licensing as part of ordinary implementation work. The repository is currently private and its existing LICENSE is proprietary; a future open-source release requires a separate authorized licensing and publication task.
- Do not add secrets, API keys, machine identifiers, private hardware identifiers, local environment files, or model weights to the repository.

## Rust workspace and crate boundaries

- Keep `crates/daedalus-system-info` portable, `no_std`, allocation-free, and independent of operating-system APIs. It owns normalized types, fixed-capacity collections, and the `SystemInfoProvider` abstraction.
- Keep host-specific API calls inside `crates/daedalus-system-discovery` or a future target-specific provider crate. Do not add `sysinfo`, OS bindings, or host-only dependencies to the portable contract crate.
- Keep protocol structures and wire compatibility in `crates/runtime-protocol`.
- Keep method dispatch and provider orchestration in `crates/runtime-engine`. Inject providers through the existing trait boundary instead of coupling the engine to platform APIs.
- Keep newline-delimited JSON stdin/stdout handling in `crates/runtime-ipc`. Stdout is reserved for protocol responses; diagnostics belong on stderr.
- Keep the `daedalus` CLI in `apps/daedalus-cli`. It invokes the engine in-process and does not contact the separate daemon.
- Keep the `daedalus-runtime` executable in `apps/runtime-daemon`. It exposes the runtime over the stdio transport.
- Keep desktop UI, Electron integration, marketplace business logic, payment processing, and hosted creator services outside this Rust runtime workspace.

## Data contract and privacy rules

- Represent unknown or unsupported readings as unavailable values. Never invent default hardware values to make a snapshot appear complete.
- Preserve the semantic distinction between `None` (a category could not be queried) and `Some(empty)` (the query succeeded but returned no entries) for optional device collections.
- Preserve fixed-capacity bounds unless a reviewed contract change justifies new memory behavior.
- Keep the hardware snapshot schema version distinct from the transport protocol version.
- Treat any wire-format change as a compatibility decision. Update protocol tests, API documentation, and README examples together.
- Do not add serial numbers, product UUIDs, asset tags, persistent hardware IDs, PCI addresses, host names, user accounts, process data, network interfaces, IP/MAC addresses, or disk mount paths to default discovery.
- Obtain explicit consent before transmitting machine-specific discovery results outside the device.
- Do not claim FIPS 140-3 validation, security certification, or compliance based only on Rust, cryptographic dependencies, or source availability.

## Implementation and verification workflow

Run commands from the workspace root:

```bash
cargo fmt --all
cargo fmt --all -- --check
cargo check --workspace
cargo build --workspace
cargo test --workspace -- --test-threads=1
```

For CLI changes, smoke-test the affected commands:

```bash
cargo run -p daedalus-cli -- hardware
cargo run -p daedalus-cli -- info
cargo run -p daedalus-cli -- ping
cargo run -p daedalus-cli -- --help
cargo run -p daedalus-cli -- --version
```

For daemon or protocol changes, test newline-delimited requests and malformed-input behavior:

```bash
printf '%s\n' '{"protocolVersion":1,"requestId":"ping-001","method":"runtime.ping","params":{}}' | cargo run -p daedalus-runtime
```

Use the narrowest crate test while iterating, then run the workspace suite before calling a Rust change complete. Hardware-facing tests should be run serially where they inspect host state. A successful compile on one OS does not establish runtime behavior on another OS or on physical hardware.

The portable contract can be checked for a no_std target separately:

```bash
rustup target add thumbv7em-none-eabihf
cargo check -p daedalus-system-info --target thumbv7em-none-eabihf
```

This does not make the host adapter, engine, CLI, or daemon bare-metal compatible.

## Documentation rules

- `../README.md` is the human-facing project guide.
- `Agents_Context.md` is short-lived working context and must be refreshed when implementation or verification status changes.
- `ARCHITECTURE.md` is the single durable architecture authority.
- `CLAUDE.md` is a minimal Claude Code import entry point.
- `../docs/SYSTEM_DISCOVERY.md` is the focused system-discovery API reference.
- Keep implemented behavior, user-reported validation, externally documented upstream support, and future plans clearly distinct.
- Update the architecture document when a durable crate boundary, protocol contract, or security invariant changes. Do not use context notes as a second architecture specification.
- Keep command examples aligned with current package names and executable names.
- Avoid documenting roadmap features as if code already implements them.
