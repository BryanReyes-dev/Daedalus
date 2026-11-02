# Daedalus — Working Context

> Short-lived project status, verified evidence, and next work. The durable architecture authority is [ARCHITECTURE.md](./ARCHITECTURE.md); refresh this document when implementation or verification status changes.

Last reviewed: 2026-11-02.

## Documentation authority

- [../README.md](../README.md) — human-facing overview, setup, CLI, daemon, tests, platform notes, and roadmap.
- [AGENTS.md](./AGENTS.md) — coding-agent workflow and implementation constraints.
- [ARCHITECTURE.md](./ARCHITECTURE.md) — durable crate boundaries, contracts, and security/privacy decisions.
- [CLAUDE.md](./CLAUDE.md) — Claude Code entry point importing the agent documents.
- [../docs/SYSTEM_DISCOVERY.md](../docs/SYSTEM_DISCOVERY.md) — detailed hardware-discovery API and field semantics.

Do not create a second architecture or implementation handoff. Put lasting decisions in ARCHITECTURE.md and refresh current status here.

## Project state

The Rust runtime foundation and system-discovery milestone is merged into `main` through pull request [#1](https://github.com/BryanReyes-dev/Daedalus/pull/1).

The merged workspace includes:
- Portable `no_std`, allocation-free system information contracts.
- Host discovery using `sysinfo 0.40.1`.
- Versioned JSON protocol and runtime dispatch.
- In-process `daedalus` CLI.
- Newline-delimited JSON stdio `daedalus-runtime` daemon.
- System-discovery API documentation and developer instructions.

Implemented protocol methods are `runtime.ping`, `runtime.getInfo`, and `hardware.discover`. Workflow execution, model inference, sandboxing, package parsing/signature verification, accelerator scheduling, and a complete embedded runtime are not implemented.

## Verification evidence

**Project-author-reported Windows 11 results, 2026-11-02:**

- `cargo fmt --all -- --check` — passed.
- `cargo check --workspace` — passed.
- `cargo test --workspace -- --test-threads=1` — passed.
- CLI smoke tests for `hardware`, `info`, `ping`, `--help`, and `--version` — reported passed.

The initial formatter run changed four Rust source files; those changes were committed and pushed as `b57c167` before PR #1 was merged.

**Not verified by this evidence:**
- Native execution on Linux, macOS, Android/iOS, BSD, or Raspberry Pi.
- GitHub Actions or other hosted CI.
- A `thumbv7em-none-eabihf` no_std target check.
- A release build, installer, or packaged binary.
- Full bare-metal execution or any board-specific provider.
- GPU compute API support, model inference, workflow execution, or cryptographic package verification.
- Production deployment or a public release.

Do not change those statuses without new observed evidence.

## Repository and licensing state

The repository is currently private. The checked-in LICENSE is proprietary / All Rights Reserved. Apache-2.0 plus DCO has been discussed as a possible public-release model, but it has not been applied. Do not claim this code is open source, publicly distributable, Apache-licensed, or contribution-ready until the owner explicitly completes the licensing and publication work.

Daedalus Desktop and Daedalus Marketplace remain separate private product areas. Keep UI and commercial marketplace logic outside this runtime workspace.

## Immediate next work

1. **Documentation structure:** keep README, agents-files, and docs aligned with the actual workspace and explicit current/future status.
2. **Native platform validation:** run discovery and test the workspace on additional target machines/OSes. Record exact OS/version/hardware and command results.
3. **GitHub Actions:** add a CI matrix only as a separately scoped task. The current milestone intentionally deferred workflows.
4. **Runtime design:** specify workflow definition/versioning, validation, lifecycle, cancellation, scheduling, and resource-budget contracts before implementing execution.
5. **Inference design:** decide model-loading, memory ownership, CPU/GPU backend abstraction, offline constraints, and failure/resource behavior before adding inference code.
6. **Security architecture:** threat-model protocol input, package parsing, model files, future WASI components, signature verification, trust roots, and update paths before implementation.
7. **Release/legal preparation:** keep the repo private and under the current license until the intended license, notices, DCO policy, repository boundary, and publication plan are explicitly approved.

## Working rules

- Start each task by checking the branch and working tree.
- Preserve user changes and avoid destructive git commands.
- Keep `daedalus-system-info` `no_std` and host-API-free.
- Keep OS-specific implementation in provider adapters.
- Preserve optional-field semantics and bounded data structures.
- Update tests and docs with protocol/data-contract changes.
- Use actual test output as evidence; do not infer platform verification from dependency support or a successful compile.
- Keep GitHub Actions deferred unless requested.
