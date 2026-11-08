# Licensing, commercial use, and product boundaries

## Public Rust runtime

The project-authored source code and documentation in this repository are licensed under the Apache License, Version 2.0. See the root `LICENSE` and `NOTICE`.

Apache-2.0 permits commercial use, internal use, modification, redistribution, and the sale of products that include the licensed work, subject to its conditions. It is a permissive license: it does not impose a general requirement to publish the source code of a larger application that uses the runtime. Recipients must receive the license and applicable notices, and modified files must be identified as required by the license.

The copyright owner retains copyright in the original work. Apache-2.0 does not grant rights to the Daedalus trademark, logos, or branding except for the limited use stated in the license.

## Product licensing boundaries

| Component | Repository | License |
|---|---|---|
| Rust runtime, host adapters, protocol, CLI, daemon, and runtime documentation | `BryanReyes-dev/Daedalus` | Apache-2.0 |
| Desktop UI, Electron shell, application integrations, and Desktop product experience | `BryanReyes-dev/Daedalus-Runtime` | Proprietary, with any explicitly identified Rust runtime subtree separately licensed under Apache-2.0 |
| Marketplace service, storefront, creator services, and distribution business logic | `BryanReyes-dev/Daedalus-Marketplace` | Proprietary |
| Marketplace skills, workflows, tools, models, and other uploaded artifacts | Each artifact's publisher | Per-artifact license |

A repository's public or private visibility does not itself determine the license. Follow the checked-in license files and the scope documented for each component.

## Commercial model

The Apache-licensed runtime can be used in commercial and public-sector solutions under Apache-2.0. Revenue may come from separately licensed Desktop functionality, Marketplace services, hosted services, custom integrations, deployment, maintenance, support, training, and contract-specific engineering.

The project cannot use this license to charge recipients for permissions that Apache-2.0 already grants to the runtime itself. It may charge for separate proprietary components and services, subject to the rights it actually owns and the terms of each applicable contract.

## Government and regulated deployments

The repository license does not decide the rights in every government contract or make a deployment compliant with a government security standard. Before contract performance:

- identify pre-existing/background software and its license;
- identify third-party dependencies, model weights, datasets, and other restricted materials;
- maintain source-control, development, and dependency/SBOM records;
- distinguish pre-existing software from contract-funded modifications and deliverables;
- review the solicitation and contract clauses with counsel before agreeing to software/data-rights terms; and
- make security and certification claims only when supported by evidence for the exact product, module, platform, and configuration.

Do not assume that public Apache-licensed code can later be made proprietary by adding a contract label. Contract rights and obligations must be assessed for the specific deliverable and agreement.

## Contributions

Contributions follow `CONTRIBUTING.md` and DCO 1.1 sign-off. DCO sign-off is a contribution-origin certification; it is not a copyright assignment or an exclusive relicensing grant. Contributions accepted under the project's public contribution policy remain available under Apache-2.0.

A separate written contributor agreement is required before accepting contributions on terms that differ from the public license.

## Third-party components

Dependencies and other third-party materials retain their own licenses. The Apache license for this repository does not automatically cover model weights, datasets, fonts, icons, images, generated assets, or external packages. Review and retain their notices separately.
