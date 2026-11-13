<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Proof API Packages

The packages in this directory are used to build components of the IBC Eureka proof API. Each package serves a specific purpose in the proof API architecture. They are put together to build a modular and extensible proof API binary in the [programs directory](../../programs/proof-api/).

The packages are organized as follows:
- `core/`: Contains the core traits and types used to build the proof API binary and modules.
- `lib/`: Contains the shared libraries used to build proof API modules.
- `modules/`: Contains modules that build proofs and unsigned transactions for a specific source and target chain type.
