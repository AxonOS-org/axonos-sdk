// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou <denis@axonos.org>
// Part of the AxonOS project — https://github.com/AxonOS-org

//! Mesh integration — AxonOS Consent Protocol client facade.
//!
//! # ⚠️ Stub status
//!
//! This module provides a **typed API surface only**. The actual protocol
//! implementation (CBOR codec, state machine, wire transport) lives in the
//! separate `axonos-consent` crate. Until kernel integration ships, all
//! methods are no-ops returning `Ok(())`.
//!
//! Do not mistake `MeshClientStub` for a working client — it is a
//! **compile-time contract** that ensures downstream code will not break
//! when the real implementation arrives.

use crate::error::Result;

/// Peer identifier — 16 bytes opaque blob.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PeerId(pub [u8; 16]);

impl PeerId {
    #[must_use]
    pub const fn from_bytes(b: [u8; 16]) -> Self {
        Self(b)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

/// Scope of consent operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "scope", rename_all = "snake_case"))]
pub enum ConsentScope {
    Peer(PeerId),
    All,
}

/// Withdraw reason per AxonOS Consent Protocol §3.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)
)]
#[repr(u8)]
pub enum WithdrawReason {
    Unspecified = 0x00,
    UserInitiated = 0x01,
    SafetyViolation = 0x02,
    HardwareFault = 0x03,
}

/// **Stub** implementation of the AxonOS mesh consent client.
///
/// # Warning
/// This is **not a real client**. All methods are no-ops until the
/// `axonos-consent` crate provides a backing implementation.
///
/// When the kernel ships, this type will be replaced by a real
/// `MeshClient` that speaks the wire protocol. The API surface
/// (method signatures) will remain identical.
#[derive(Debug)]
pub struct MeshClientStub {
    session_id: u64,
}

impl MeshClientStub {
    #[must_use]
    pub const fn new(session_id: u64) -> Self {
        Self { session_id }
    }

    #[must_use]
    pub const fn session_id(&self) -> u64 {
        self.session_id
    }

    /// Request a consent-withdraw frame.
    ///
    /// # Current behavior
    /// No-op. Returns `Ok(())` unconditionally.
    pub fn withdraw_consent(&self, scope: ConsentScope, reason: WithdrawReason) -> Result<()> {
        let _ = (scope, reason);
        Ok(())
    }

    /// Request a consent-suspend frame.
    ///
    /// # Current behavior
    /// No-op. Returns `Ok(())` unconditionally.
    pub fn suspend_consent(&self, scope: ConsentScope) -> Result<()> {
        let _ = scope;
        Ok(())
    }

    /// Request a consent-resume frame.
    ///
    /// # Current behavior
    /// No-op. Returns `Ok(())` unconditionally.
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
        assert_eq!(WithdrawReason::Unspecified as u8, 0x00);
        assert_eq!(WithdrawReason::UserInitiated as u8, 0x01);
        assert_eq!(WithdrawReason::SafetyViolation as u8, 0x02);
        assert_eq!(WithdrawReason::HardwareFault as u8, 0x03);
    }

    #[test]
    fn stub_has_session_id() {
        let c = MeshClientStub::new(0xDEAD_BEEF);
        assert_eq!(c.session_id(), 0xDEAD_BEEF);
    }
}

