# Daedalus

**Daedalus is the public desktop application repository for a local-first AI workflow platform.**

This repository is intended to contain the desktop user experience and application shell: the Next.js interface, the Electron host (with a possible future Tauri host), the renderer-to-host bridge, and the client-side integrations that connect the application to the Daedalus runtime and marketplace.

The Rust execution runtime and the hosted marketplace are maintained separately:

| Repository | Visibility | Responsibility |
|---|---|---|
| **[Daedalus](https://github.com/BryanReyes-dev/Daedalus)** | Public | Desktop UI, application shell, user-facing workflows, host adapters, and client integrations |
| **[Daedalus-Runtime](https://github.com/BryanReyes-dev/Daedalus-Runtime)** | Private | Rust runtime, workflow execution, model and archive handling, sandboxing, and runtime-side cryptographic operations |
| **[Daedalus-Marketplace](https://github.com/BryanReyes-dev/Daedalus-Marketplace)** | Private | Storefront, creator services, artifact storage, licensing, payments, and payouts |

> **Repository status:** This repository currently contains its project README. The application source has not yet been migrated here. The existing Next.js/Electron/Rust codebase and the initial Rust foundation are still in the private `Daedalus-Runtime` repository and must be separated deliberately before this repository can be built as the desktop application.

## Responsibilities and boundaries

### This repository: Daedalus desktop application

The public application repository is responsible for:

- **User interface:** Next.js-based desktop interface, workflow authoring experience, settings, and user-facing status.
- **Desktop host:** Electron main process and preload bridge; the host can be replaced by Tauri later without moving runtime responsibilities into the UI.
- **Runtime integration:** A narrow, versioned client contract and host adapter for starting the runtime and exchanging requests/events.
- **Marketplace integration:** User-facing sign-in, catalog, purchase, entitlement, and download flows that call the private marketplace service.
- **Application experience:** Local preferences, presentation, onboarding, and orchestration of user-initiated actions.

The renderer must not directly own workflow execution, model memory, privileged operating-system access, or cryptographic secrets. It should call narrowly scoped host capabilities, and the host should communicate with the runtime through a versioned local protocol.

### Private repository: Daedalus Runtime

The private runtime repository owns the execution engine and all runtime-critical behavior, including the planned:

- Portable workflow/DAG data structures and validation.
- Workflow scheduling, execution state, cancellation, and resource arbitration.
- `.dea` archive parsing, integrity checks, and package loading.
- Model lifecycle and in-process inference.
- Wasmtime/WASI guest execution and resource limits.
- Runtime-side cryptographic verification, decryption interfaces, and key handling.
- Hardware/system discovery and platform-specific runtime capabilities.
- Headless runtime and CLI interfaces.

The desktop application should consume the runtime through a stable protocol or a deliberately versioned integration package. It should not copy or reimplement the runtime engine.

### Private repository: Daedalus Marketplace

The private marketplace repository owns hosted commercial services, including:

- Storefront and creator-facing workflows.
- PostgreSQL-backed catalog, creator, purchase, and entitlement records.
- Encrypted artifact storage and controlled distribution.
- License issuance and verification services.
- Payment processing and creator payouts.
- Release distribution and associated server-side operations.

The marketplace is the authority for commercial entitlements and payment state. The desktop client presents those flows; it must not contain marketplace master secrets or signing private keys.

## Security and trust boundaries

This repository is public. Treat every client-side component as inspectable and potentially modifiable by the person running it.

- Never commit API secrets, marketplace signing keys, private decryption keys, production credentials, or customer data.
- Do not rely on Electron, JavaScript, local hardware identifiers, or client-side checks as the sole enforcement of a commercial entitlement.
- Keep authoritative license issuance and signing operations server-side. The runtime should validate signed, scoped authorization material according to the protocol agreed with the marketplace.
- Assume that plaintext data made available on a user's machine—including decrypted package contents in memory—may be inspected by a sufficiently capable user with control of that machine.
- Keep renderer permissions narrow: use context isolation, disable Node integration, expose explicit preload methods, validate IPC senders, and validate all runtime and marketplace inputs.
- Do not describe hardware binding or client-side DRM as unbreakable copy protection. These controls can raise the cost of abuse, but they cannot make a user-controlled endpoint a trusted execution environment.

The private runtime is not publicly auditable through this repository. That is a deliberate consequence of the current repository-visibility decision and differs from an architecture where the core execution engine is open source.

## Intended application layout

The exact layout will be established during migration. The expected separation is:

```text
daedalus/
├── apps/
│   └── desktop-ui/
│       ├── ui/                 # Next.js renderer
│       └── electron/            # Electron main process and preload
├── packages/
│   └── runtime-client/          # Framework-neutral runtime contract
├── scripts/                     # Development and release orchestration
├── docs/                        # Application and integration documentation
└── README.md
```

This repository should not contain the private runtime's Cargo workspace, execution-engine implementation, marketplace backend, database schema, or signing infrastructure. Small host-side adapters and protocol definitions may live here when they are specifically part of the public application integration.

## Development status

The repository is not yet ready for local application development because the application source and the runtime distribution boundary have not been migrated.

The migration should establish all of the following before declaring the desktop app buildable:

1. The Next.js/Electron application source exists in this repository.
2. The runtime is consumed through a pinned, versioned interface; the private runtime source is not copied into this repository.
3. Local development has a documented runtime option, such as an authorized private runtime checkout or a development stub.
4. Release builds obtain a matching, signed runtime binary/artifact without embedding credentials in the public source.
5. Marketplace calls target documented APIs and do not expose server secrets.
6. CI verifies the desktop application, its runtime compatibility, and its release artifact provenance.

Until those steps are complete, there are no reliable install, build, or run commands to document here.

## Licensing

This is a public source repository, but public visibility alone does not grant permission to reuse, redistribute, or sell its contents. A license has not yet been specified in this repository; add the intended `LICENSE` file before accepting outside contributions or advertising reuse rights.

## Related repositories

- [Daedalus Runtime](https://github.com/BryanReyes-dev/Daedalus-Runtime) — private Rust execution runtime.
- [Daedalus Marketplace](https://github.com/BryanReyes-dev/Daedalus-Marketplace) — private hosted marketplace and licensing services.
