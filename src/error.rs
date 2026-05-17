// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou <denis@axonos.org>
// Part of the AxonOS project — https://github.com/AxonOS-org

//! Error taxonomy for the AxonOS SDK.
//!
//! Layered per IEC 62304 §5.2.6:
//! - L1 — Transport
//! - L2 — Capability/quota
//! - L3 — Consent state
//! - L4 — Protocol/wire format

#[cfg(not(feature = "std"))]
use core::fmt;

/// Result alias.
pub type Result<T> = core::result::Result<T, Error>;

/// Top-level error enum.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
#[non_exhaustive]
pub enum Error {
    // L1 — Transport
    #[cfg_attr(feature = "std", error("kernel transport unreachable: {0:?}"))]
    TransportUnreachable(TransportFault),

    #[cfg_attr(
        feature = "std",
        error("kernel ABI mismatch: sdk={sdk}, kernel={kernel}")
    )]
    AbiMismatch { sdk: u32, kernel: u32 },

    // L2 — Capability/quota
    #[cfg_attr(feature = "std", error("capability {0:?} not declared in manifest"))]
    CapabilityNotDeclared(crate::Capability),

    #[cfg_attr(feature = "std", error("manifest rejected: {reason:?}"))]
    ManifestRejected { reason: ManifestRejection },

    #[cfg_attr(
        feature = "std",
        error("rate limit exceeded: declared={max_rate_hz} Hz")
    )]
    RateLimitExceeded { max_rate_hz: u32 },

    // L3 — Consent
    #[cfg_attr(feature = "std", error("consent suspended"))]
    ConsentSuspended,

    #[cfg_attr(feature = "std", error("consent withdrawn"))]
    ConsentWithdrawn,

    // L4 — Protocol
    #[cfg_attr(feature = "std", error("protocol parse error: {0:?}"))]
    Protocol(ProtocolFault),

    #[cfg_attr(feature = "std", error("attestation verification failed"))]
    AttestationFailed,

    #[cfg_attr(
        feature = "std",
        error("stream buffer overflow: {dropped} observations dropped")
    )]
    StreamOverflow { dropped: u32 },

    // Other
    #[cfg(feature = "std")]
    #[cfg_attr(feature = "std", error("I/O error: {0}"))]
    Io(String),
}

impl Error {
    /// True if terminal — subscription must be torn down.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::ConsentWithdrawn
                | Self::AbiMismatch { .. }
                | Self::ManifestRejected { .. }
                | Self::AttestationFailed
        )
    }

    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::TransportUnreachable(_) => ErrorCode::TransportUnreachable,
            Self::AbiMismatch { .. } => ErrorCode::AbiMismatch,
            Self::CapabilityNotDeclared(_) => ErrorCode::CapabilityNotDeclared,
            Self::ManifestRejected { .. } => ErrorCode::ManifestRejected,
            Self::RateLimitExceeded { .. } => ErrorCode::RateLimitExceeded,
            Self::ConsentSuspended => ErrorCode::ConsentSuspended,
            Self::ConsentWithdrawn => ErrorCode::ConsentWithdrawn,
            Self::Protocol(_) => ErrorCode::Protocol,
            Self::AttestationFailed => ErrorCode::AttestationFailed,
            Self::StreamOverflow { .. } => ErrorCode::StreamOverflow,
            #[cfg(feature = "std")]
            Self::Io(_) => ErrorCode::Io,
        }
    }
}

#[cfg(not(feature = "std"))]
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TransportUnreachable(f0) => write!(f, "kernel transport unreachable: {f0:?}"),
            Self::AbiMismatch { sdk, kernel } => {
                write!(f, "kernel ABI mismatch: sdk={sdk}, kernel={kernel}")
            }
            Self::CapabilityNotDeclared(c) => write!(f, "capability {c:?} not declared"),
            Self::ManifestRejected { reason } => write!(f, "manifest rejected: {reason:?}"),
            Self::RateLimitExceeded { max_rate_hz } => {
                write!(f, "rate limit exceeded: {max_rate_hz} Hz declared")
            }
            Self::ConsentSuspended => write!(f, "consent suspended"),
            Self::ConsentWithdrawn => write!(f, "consent withdrawn"),
            Self::Protocol(p) => write!(f, "protocol error: {p:?}"),
            Self::AttestationFailed => write!(f, "attestation verification failed"),
            Self::StreamOverflow { dropped } => write!(f, "stream overflow: {dropped} dropped"),
        }
    }
}

/// Machine-readable error code. Not a dense index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
#[non_exhaustive]
pub enum ErrorCode {
    TransportUnreachable = 0x0101,
    AbiMismatch = 0x0102,
    CapabilityNotDeclared = 0x0201,
    ManifestRejected = 0x0202,
    RateLimitExceeded = 0x0203,
    ConsentSuspended = 0x0301,
    ConsentWithdrawn = 0x0302,
    Protocol = 0x0401,
    AttestationFailed = 0x0402,
    StreamOverflow = 0x0403,
    Io = 0x0501,
}

/// Transport fault reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransportFault {
    EndpointNotFound,
    PermissionDenied,
    ConnectionRefused,
    Disconnected,
    Timeout,
    /// Internal state corruption (e.g., poisoned mutex).
    /// The SDK refuses to proceed when its synchronisation primitives
    /// are in an indeterminate state.
    Internal,
}

/// Manifest rejection reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ManifestRejection {
    InvalidSignature,
    ProhibitedCapability,
    RateTooHigh,
    Malformed,
    DuplicateAppId,
}

/// Wire-format protocol errors.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProtocolFault {
    TruncatedHeader,
    TruncatedBody,
    UnknownFrameType(u16),
    MissingField(&'static str),
    InvalidFieldType(&'static str),
    FrameTooLarge { size: u32, max: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_is_terminal_matches_spec() {
        assert!(Error::ConsentWithdrawn.is_terminal());
        assert!(!Error::ConsentSuspended.is_terminal());
        assert!(Error::AttestationFailed.is_terminal());
        assert!(!Error::StreamOverflow { dropped: 5 }.is_terminal());
    }

    #[test]
    fn error_codes_are_stable() {
        assert_eq!(Error::ConsentWithdrawn.code() as u16, 0x0302);
        assert_eq!(Error::ConsentSuspended.code() as u16, 0x0301);
        assert_eq!(Error::AttestationFailed.code() as u16, 0x0402);
    }
}
