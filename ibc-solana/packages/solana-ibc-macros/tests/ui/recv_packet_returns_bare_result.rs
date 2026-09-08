// SPDX-License-Identifier: Apache-2.0

use solana_ibc_macros::ibc_app;

// on_recv_packet returning bare Result (no generic args, like anchor_lang::Result alias)
#[ibc_app]
pub mod my_app {
    pub fn on_recv_packet(ctx: Ctx, msg: OnRecvPacketMsg) -> Result {
        Ok(())
    }

    pub fn on_acknowledgement_packet(ctx: Ctx, msg: OnAcknowledgementPacketMsg) -> Result<()> {
        Ok(())
    }

    pub fn on_timeout_packet(ctx: Ctx, msg: OnTimeoutPacketMsg) -> Result<()> {
        Ok(())
    }
}

fn main() {}
