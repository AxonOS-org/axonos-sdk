// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Mesh coupling: demonstrate consent withdrawal (MMP §3.1).
//!
//! ```sh
//! cargo run --example mesh_coupling --features "std serde kernel-stub"
//! ```

use axonos_sdk::mesh::{ConsentScope, MeshClientStub, PeerId, WithdrawReason};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session_id: u64 = 0xCAFE_BABE_DEAD_BEEF;
    let mesh = MeshClientStub::new(session_id);
    println!("MeshClientStub for session {:#018x}", mesh.session_id());

    let peer_x = PeerId::from_bytes([
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
    ]);

    println!("\nWithdrawing consent for peer {:x?}...", peer_x.as_bytes());
    mesh.withdraw_consent(ConsentScope::Peer(peer_x), WithdrawReason::UserInitiated)?;
    println!(" ✓ consent-withdraw frame enqueued (stub)");

    println!("\nSuspending consent for all peers...");
    mesh.suspend_consent(ConsentScope::All)?;
    println!(" ✓ consent-suspend frame enqueued (stub)");

    println!("\nResuming consent for all peers...");
    mesh.resume_consent(ConsentScope::All)?;
    println!(" ✓ consent-resume frame enqueued (stub)");

    println!("\nSafety-triggered withdrawal...");
    mesh.withdraw_consent(ConsentScope::All, WithdrawReason::HardwareFault)?;
    println!(" ✓ consent-withdraw(HARDWARE_FAULT) emitted (stub)");

    Ok(())
}
