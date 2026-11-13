# Upgradeability

This repository houses both Solidity contracts and Solana programs for IBC v2, designed with upgradeability in mind.

## Solidity Contracts

The core IBC contracts utilize the UUPS proxy pattern, ensuring controlled and efficient upgrades, while contracts deployed by the core IBC contracts follow the beacon proxy pattern for streamlined management and scalability.

| **Contract** | **Authority** | **Proxy Pattern** | **Upgrade Function** |
|:---:|:---:|:---:|:---:|
| `ICS26Router.sol` | [Admins](./README.md#security-assumptions) | [UUPSUpgradeable](https://docs.openzeppelin.com/contracts/5.x/api/proxy#UUPSUpgradeable) | `ICS26Router::upgradeToAndCall` |
| `ICS20Transfer.sol` | [Admins](./README.md#security-assumptions) | [UUPSUpgradeable](https://docs.openzeppelin.com/contracts/5.x/api/proxy#UUPSUpgradeable) | `ICS20Transfer::upgradeToAndCall` |
| `Escrow.sol` | `ICS20Transfer` | [Beacon](https://docs.openzeppelin.com/contracts/5.x/api/proxy#BeaconProxy) | `ICS20Transfer::upgradeEscrowTo` |
| `IBCERC20.sol` | `ICS20Transfer` | [Beacon](https://docs.openzeppelin.com/contracts/5.x/api/proxy#BeaconProxy) | `ICS20Transfer::upgradeIBCERC20To` |
| `ICS27GMP.sol` | [Admins](./README.md#security-assumptions) | [UUPSUpgradeable](https://docs.openzeppelin.com/contracts/5.x/api/proxy#UUPSUpgradeable) | `ICS27GMP::upgradeToAndCall` |
| `ICS27Account.sol` | `ICS27GMP` | [Beacon](https://docs.openzeppelin.com/contracts/5.x/api/proxy#BeaconProxy) | `ICS27GMP::upgradeAccountTo` |
| `IBCMulticall.sol` | [Admins](./README.md#security-assumptions) | [UUPSUpgradeable](https://docs.openzeppelin.com/contracts/5.x/api/proxy#UUPSUpgradeable) | `IBCMulticall::upgradeToAndCall` |

![Light Mode Diagram](./docs/assets/upgradeability-light.svg#gh-light-mode-only)![Dark Mode Diagram](./docs/assets/upgradeability-dark.svg#gh-dark-mode-only)

## Solana Programs

Solana programs are deployed using the [BPF Loader Upgradeable](https://solana.com/docs/core/programs#loader-programs), which allows the program's executable code to be upgraded while preserving existing program-derived addresses (PDAs) and account data.

### Upgrade Authority

The upgrade authority is set when deploying a program with the BPF Loader Upgradeable and controls who can upgrade the program. The default authority is the account which [initially deployed](https://solana.com/docs/core/programs#updating-solana-programs) the program. The authority [can always be transferred](https://solana.com/docs/programs/deploying#transfer-program-authority) to some other account.

In production, the deployer transfers each program's upgrade authority to the [Access Manager](./ibc-solana/programs/access-manager/README.md)'s per-program PDA (`["upgrade_authority", program_id]`). From that point, upgrades go through the Access Manager's role-gated `upgrade_program` instruction.

#### Upgrade Authority Transfer

Transferring upgrade authority away from the Access Manager uses a two-step propose/accept flow:

1. An admin calls `propose_upgrade_authority_transfer` specifying the target program and new authority
2. The new authority signs `accept_upgrade_authority_transfer` to execute the BPF Loader `SetAuthority` CPI

Multiple transfers can be proposed concurrently (one per target program, up to 16). A proposal can be cancelled by an admin before acceptance.

For AM-to-AM migration, the destination Access Manager calls `claim_upgrade_authority` which CPIs into the source AM's accept instruction, signing with its own PDA.

See the [Access Manager README](./ibc-solana/programs/access-manager/README.md) for detailed flows and security considerations.

### Account State Versioning

All persistent account structures include version fields (using type-safe enums) and reserved space to support future upgrades:

**ics26-router:**
- `RouterState`: `AccountVersion` enum (V1) + 256 bytes reserved space
- `Client`: `AccountVersion` enum (V1) + 256 bytes reserved space
- `IBCApp`: `AccountVersion` enum (V1) + 256 bytes reserved space

This versioning strategy allows:
- **Backward compatibility**: Old accounts can be identified and migrated
- **Future expansion**: New fields can be added using reserved space
- **Safe upgrades**: Program code can check version and handle different account layouts
