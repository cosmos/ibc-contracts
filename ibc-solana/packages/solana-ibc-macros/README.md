# solana-ibc-macros

Procedural macros for IBC applications on Solana.

## Overview

This crate provides the `#[ibc_app]` macro, which wraps Anchor's `#[program]` and validates at compile time that IBC applications implement all required callback functions with correct signatures. It also provides the `discriminator!` macro for computing Anchor instruction discriminators at compile time.

## Usage

```rust
use solana_ibc_macros::ibc_app;

#[ibc_app]
pub mod my_ibc_app {
    use super::*;

    pub fn on_recv_packet(
        ctx: Context<OnRecvPacket>,
        msg: OnRecvPacketMsg,
    ) -> Result<Vec<u8>> {
        // Handle received packet
        Ok(vec![])
    }

    pub fn on_acknowledgement_packet(
        ctx: Context<OnAckPacket>,
        msg: OnAcknowledgementPacketMsg,
    ) -> Result<()> {
        // Handle acknowledgement
        Ok(())
    }

    pub fn on_timeout_packet(
        ctx: Context<OnTimeoutPacket>,
        msg: OnTimeoutPacketMsg,
    ) -> Result<()> {
        // Handle timeout
        Ok(())
    }
}
```

## Features

- Compile-time validation of IBC callback function names and signatures
- Compile-time check that the Anchor instruction discriminators for each callback exist
- Clear error messages for missing or misnamed callbacks
