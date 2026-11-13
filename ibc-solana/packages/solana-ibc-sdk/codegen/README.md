# solana-ibc-sdk-codegen

Generates Rust types, accounts, events and instruction builders from Anchor IDL files for the `solana-ibc-sdk` crate.

Used as a build dependency — `build.rs` invokes `generate_all` to produce the `src/generated/` modules at compile time.

## Generated modules

Each program IDL produces a directory under `src/generated/{program}/` with a `mod.rs` and up to four files:

| File | Contents |
|------|----------|
| `types.rs` | Structs and enums that are not accounts or events |
| `accounts.rs` | Account structs with `DISCRIMINATOR` constants |
| `events.rs` | Event structs marked with `#[event]` |
| `instructions.rs` | Instruction builders with PDA derivation helpers |

## Type naming

Types are named using the **short name** — the last `::` segment of the IDL's fully-qualified name, converted to `PascalCase`.

```text
IDL name:       solana_ibc_types::router::Packet
Generated name: Packet

IDL name:       ics26_router::events::send_packet_event
Generated name: SendPacketEvent
```

This keeps consumer code clean:

```rust
use solana_ibc_sdk::ics26_router::types::{Packet, MsgRecvPacket, PayloadMetadata};
use solana_ibc_sdk::ics07_tendermint::accounts::ClientState;
```

### Collision handling

If two types within the same IDL share a short name, the codegen falls back to a fully-qualified form where each `::` segment is `PascalCase`-d and joined with `_`:

```text
IDL has both:
  mod_a::Shared
  mod_b::Shared

Short name "Shared" collides, so they become:
  ModA_Shared
  ModB_Shared
```

This is deterministic and requires no manual configuration. For example, the IFT IDL contains both `ift::state::AccountVersion` and `ics27_gmp::state::AccountVersion`, which generates `Ift_State_AccountVersion` and `Ics27Gmp_State_AccountVersion`.

## Examples

### Using generated types

```rust
use solana_ibc_sdk::ics26_router::{
    accounts::{IBCApp, RouterState},
    instructions::{RecvPacket, RecvPacketAccounts},
    types::{MsgRecvPacket, Packet, Payload},
};

// Deserialize an on-chain account
let router_state = RouterState::deserialize(&mut data)?;

// Build an instruction using the builder pattern
let ix = RecvPacket::builder(&program_id)
    .accounts(RecvPacketAccounts { /* ... */ })
    .args(&msg)
    .remaining_accounts(extra_accounts)
    .build();
```

### Using PDA helpers

Each instruction exposes static methods for deriving PDAs:

```rust
use solana_ibc_sdk::ics26_router::instructions::SendPacket;

let (client_pda, bump) = SendPacket::client_pda(client_id, &program_id);
let (ibc_app_pda, bump) = SendPacket::ibc_app_pda(port_id, &program_id);
```

## Running tests

```bash
cd ibc-solana
cargo test -p solana-ibc-sdk-codegen
cargo clippy -p solana-ibc-sdk-codegen --lib --tests
```
