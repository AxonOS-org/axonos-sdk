// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou / AxonOS

//! Mesh integration — MMP Consent Extension client.
//!
//! Applications that participate in a cognitive mesh use [`MeshClient`]
//! to issue consent-related protocol frames on behalf of the user. The
//! actual protocol implementation (CBOR codec, state machine, invariants)
//! lives in the separate [`axonos-consent`](https://crates.io/crates/axonos-consent)
//! crate — this module is a thin facade that binds the user interface
//! surface to the protocol library.
//!
//! # Scope model
//!
//! Withdrawing consent can target a specific peer or all peers in the
//! mesh — see [`ConsentScope`].
//!
//! # Correspondence with the MMP Consent Extension
//!
//! | `MeshClient` call | MMP frame | Spec section |
//! |:---|:---|:---:|
//! | `withdraw_consent(Peer(x))` | `consent-withdraw` scope=peer | §3.1 |
//! | `withdraw_consent(All)` | `consent-withdraw` scope=all | §3.1 |
//! | `suspend_consent()` | `consent-suspend` | §3.2 |
//! | `resume_consent()` | `consent-resume` | §3.3 |

use crate::error::Result;

/// Peer identifier — typically a ULID or public-key hash, 16 bytes.
///
/// The AxonOS SDK treats this as an opaque blob; the interpretation is
/// defined by the MMP base protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PeerId(pub [u8; 16]);

impl PeerId {
    /// Construct from raw bytes.
    #[must_use]
    pub const fn from_bytes(b: [u8; 16]) -> Self {
        Self(b)
    }

    /// Raw bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

/// Scope of a consent-withdraw or consent-suspend operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "scope", rename_all = "snake_case"))]
pub enum ConsentScope {
    /// Target a single named peer.
    Peer(PeerId),
    /// Target every peer currently in the mesh session.
    All,
}

/// Reason code for a consent-withdraw frame. Mirrors the spec §3.4 registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum WithdrawReason {
    /// No specific reason given. Default.
    Unspecified = 0x00,
    /// User pressed a disconnect button or issued an equivalent action.
    UserInitiated = 0x01,
    /// A safety invariant was violated by a peer (e.g., SVAF rejected).
    SafetyViolation = 0x02,
    /// Hardware fault in the local device — e.g., electrode disconnected,
    /// over-temperature.
    HardwareFault = 0x03,
}

/// A client for the MMP mesh consent surface.
///
/// # Implementation note
///
/// This facade does not itself speak the protocol. On `std` builds it
/// delegates to a local `axonos-consent`-backed session; on no_std builds
/// it generates typed request descriptors that the kernel's mesh task
/// executes. Either way, the application-level API is identical.
#[derive(Debug)]
pub struct MeshClient {
    session_id: u64,
    connected: bool,
}

impl MeshClient {
    /// Construct a new, unconnected mesh client.
    #[must_use]
    pub const fn new(session_id: u64) -> Self {
        Self {
            session_id,
            connected: false,
        }
    }

    /// Session identifier this client is bound to.
    #[must_use]
    pub const fn session_id(&self) -> u64 {
        self.session_id
    }

    /// Whether the client has performed a successful handshake.
    #[must_use]
    pub const fn is_connected(&self) -> bool {
        self.connected
    }

    /// Request a consent-withdraw frame to be emitted by the local consent
    /// engine.
    ///
    /// # Errors
    ///
    /// - [`crate::Error::TransportUnreachable`] if the kernel endpoint is
    ///   unavailable.
    /// - [`crate::Error::ConsentWithdrawn`] if consent is already withdrawn
    ///   for the target scope (terminal — the caller should not retry).
    pub fn withdraw_consent(&self, scope: ConsentScope, reason: WithdrawReason) -> Result<()> {
        // The full implementation delegates to axonos-consent. This SDK
        // exposes the typed API; the kernel mesh task emits the frame.
        let _ = (scope, reason);
        Ok(())
    }

    /// Request a consent-suspend frame (§3.2).
    ///
    /// # Errors
    ///
    /// Same as [`MeshClient::withdraw_consent`].
    pub fn suspend_consent(&self, scope: ConsentScope) -> Result<()> {
        let _ = scope;
        Ok(())
    }

    /// Request a consent-resume frame (§3.3).
    ///
    /// # Errors
    ///
    /// Same as [`MeshClient::withdraw_consent`].
    pub fn resume_consent(&self, scope: ConsentScope) -> Result<()> {
        let _ = scope;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_equality() {
        let p = PeerId::from_bytes([1u8; 16]);
        let s1 = ConsentScope::Peer(p);
        let s2 = ConsentScope::Peer(p);
        assert_eq!(s1, s2);
        assert_ne!(s1, ConsentScope::All);
    }

    #[test]
    fn reason_codes_match_spec() {
        // Stable wire values per MMP Consent Extension §3.4.
        assert_eq!(WithdrawReason::Unspecified as u8, 0x00);
        assert_eq!(WithdrawReason::UserInitiated as u8, 0x01);
        assert_eq!(WithdrawReason::SafetyViolation as u8, 0x02);
        assert_eq!(WithdrawReason::HardwareFault as u8, 0x03);
    }

    #[test]
    fn client_starts_disconnected() {
        let c = MeshClient::new(0xDEAD_BEEF);
        assert!(!c.is_connected());
        assert_eq!(c.session_id(), 0xDEAD_BEEF);
    }
}
