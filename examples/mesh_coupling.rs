// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Mesh coupling example: connect to a cognitive mesh, then demonstrate
//! consent withdrawal (MMP Consent Extension §3.1).
//!
//! Run with:
//!
//! ```sh
//! cargo run --example mesh_coupling --features "std serde"
//! ```

use axonos_sdk::mesh::{ConsentScope, MeshClient, PeerId, WithdrawReason};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // In a real application, session_id comes from the kernel handshake.
    let session_id: u64 = 0xCAFE_BABE_DEAD_BEEF;

    let mesh = MeshClient::new(session_id);
    println!("Mesh client constructed for session {:#018x}", mesh.session_id());

    // Scenario 1: The user presses the "disconnect from peer X" button.
    let peer_x = PeerId::from_bytes([
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10,
    ]);

    println!();
    println!("Withdrawing consent for peer {:x?}...", peer_x.as_bytes());
    mesh.withdraw_consent(ConsentScope::Peer(peer_x), WithdrawReason::UserInitiated)?;
    println!("  ✓ consent-withdraw frame enqueued for emission");

    // Scenario 2: User presses "suspend coupling with everyone" (focus mode).
    println!();
    println!("Suspending consent for all peers (focus mode)...");
    mesh.suspend_consent(ConsentScope::All)?;
    println!("  ✓ consent-suspend frame enqueued");

    // Scenario 3: User resumes coupling.
    println!();
    println!("Resuming consent for all peers...");
    mesh.resume_consent(ConsentScope::All)?;
    println!("  ✓ consent-resume frame enqueued");

    // Scenario 4: Safety violation — hardware reports over-temperature.
    println!();
    println!("Safety-triggered withdrawal (over-temperature)...");
    mesh.withdraw_consent(ConsentScope::All, WithdrawReason::HardwareFault)?;
    println!("  ✓ consent-withdraw(HARDWARE_FAULT) emitted to all peers");

    Ok(())
}
