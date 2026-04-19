// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou / AxonOS

//! Error taxonomy for the AxonOS SDK.
//!
//! All fallible operations return [`Result<T>`], which is an alias for
//! `core::result::Result<T, Error>`. The error enum is exhaustive — adding
//! a new variant is a breaking change and will require a major version bump.
//!
//! The taxonomy is layered:
//! - **L1 — Transport errors**: the kernel boundary could not be reached.
//! - **L2 — Capability errors**: the application exceeded its declared
//!   capabilities or quota.
//! - **L3 — Consent errors**: the user has suspended or withdrawn consent.
//! - **L4 — Protocol errors**: the wire format is malformed or the ABI
//!   version is incompatible.
//!
//! This layering matches the error model in `axonos-consent` and follows
//! the IEC 62304 §5.2.6 requirement that error handling be explicit and
//! traceable.

use core::fmt;

/// Result type alias used throughout the SDK.
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Top-level error enum for all SDK operations.
///
/// Variants are grouped by layer (see module documentation).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
#[non_exhaustive]
pub enum Error {
    // ─── L1 — Transport / kernel boundary ────────────────────────────
    /// The kernel IPC endpoint is not reachable. Typically indicates a
    /// driver or permissions issue in the hosting process.
    #[cfg_attr(feature = "std", error("kernel transport unreachable: {0:?}"))]
    TransportUnreachable(TransportFault),

    /// Kernel ABI version does not match [`crate::KERNEL_ABI_VERSION`].
    /// The application was built against a different AxonOS SDK major
    /// version than the kernel it is trying to connect to.
    #[cfg_attr(feature = "std", error("kernel ABI mismatch: sdk={sdk}, kernel={kernel}"))]
    AbiMismatch {
        /// SDK version.
        sdk: u32,
        /// Kernel version.
        kernel: u32,
    },

    // ─── L2 — Capability / quota ─────────────────────────────────────
    /// The requested capability is not declared in the application manifest.
    #[cfg_attr(feature = "std", error("capability {0:?} not declared in manifest"))]
    CapabilityNotDeclared(crate::Capability),

    /// The application's manifest was rejected by the kernel. This happens
    /// when the manifest requests a [`crate::Capability`] that the kernel
    /// policy prohibits (e.g., `RawEeg`) or when the signature verification
    /// fails.
    #[cfg_attr(feature = "std", error("manifest rejected: {reason:?}"))]
    ManifestRejected {
        /// Human-readable reason, intended for developer logs only.
        reason: ManifestRejection,
    },

    /// The application exceeded its declared max event rate.
    #[cfg_attr(feature = "std", error("rate limit exceeded: declared={max_rate_hz} Hz"))]
    RateLimitExceeded {
        /// Declared rate.
        max_rate_hz: u32,
    },

    // ─── L3 — Consent state ──────────────────────────────────────────
    /// The user has suspended consent — no new intent events will be delivered
    /// until consent is resumed. The stream remains open; this is not a fatal
    /// error.
    #[cfg_attr(feature = "std", error("consent suspended"))]
    ConsentSuspended,

    /// The user has withdrawn consent. The stream is terminated. This is
    /// terminal for the session — the application must close its subscription
    /// and, if desired, request a fresh handshake. This corresponds to the
    /// MMP Consent Extension WITHDRAWN state, which is terminal.
    #[cfg_attr(feature = "std", error("consent withdrawn"))]
    ConsentWithdrawn,

    // ─── L4 — Protocol / wire format ─────────────────────────────────
    /// The wire frame could not be parsed. Wraps a specific parse fault.
    #[cfg_attr(feature = "std", error("protocol parse error: {0:?}"))]
    Protocol(ProtocolFault),

    /// The observation HMAC did not verify against the session key. The
    /// event is discarded and the application should log this as a
    /// potential tampering incident.
    #[cfg_attr(feature = "std", error("attestation verification failed"))]
    AttestationFailed,

    /// The stream buffer overflowed and observations were dropped. The
    /// count reflects the number of observations lost since the last
    /// successful poll.
    #[cfg_attr(feature = "std", error("stream buffer overflow: {dropped} observations dropped"))]
    StreamOverflow {
        /// Number of dropped observations.
        dropped: u32,
    },

    // ─── Other ────────────────────────────────────────────────────────
    /// I/O error from the host platform (std builds only).
    #[cfg(feature = "std")]
    #[cfg_attr(feature = "std", error("I/O error: {0}"))]
    Io(String),
}

impl Error {
    /// Returns `true` if this error is terminal for the current session —
    /// i.e., the application must tear down its subscription. Non-terminal
    /// errors can be recovered from by retrying or waiting.
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

    /// Returns a stable, machine-readable error code for logging and
    /// telemetry. Codes are compatible with the `axonos-consent` reason
    /// code registry where applicable.
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

// Manual Display impl for no_std builds.
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

/// Machine-readable error code, suitable for logging, metrics, and wire
/// protocol error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum ErrorCode {
    /// L1 — transport unreachable.
    TransportUnreachable = 0x0101,
    /// L1 — ABI mismatch.
    AbiMismatch = 0x0102,
    /// L2 — capability not declared.
    CapabilityNotDeclared = 0x0201,
    /// L2 — manifest rejected by kernel policy.
    ManifestRejected = 0x0202,
    /// L2 — rate limit exceeded.
    RateLimitExceeded = 0x0203,
    /// L3 — consent suspended.
    ConsentSuspended = 0x0301,
    /// L3 — consent withdrawn.
    ConsentWithdrawn = 0x0302,
    /// L4 — protocol parse error.
    Protocol = 0x0401,
    /// L4 — attestation failed.
    AttestationFailed = 0x0402,
    /// L4 — stream overflow.
    StreamOverflow = 0x0403,
    /// Host I/O error (std only).
    Io = 0x0501,
}

/// Specific reasons a transport request can fail.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransportFault {
    /// The IPC endpoint file/device does not exist.
    EndpointNotFound,
    /// Permission denied opening the endpoint.
    PermissionDenied,
    /// The connection was refused by the kernel.
    ConnectionRefused,
    /// The connection was closed unexpectedly.
    Disconnected,
    /// A timeout elapsed waiting for kernel response.
    Timeout,
}

/// Specific reasons a manifest can be rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ManifestRejection {
    /// The manifest signature did not verify.
    InvalidSignature,
    /// A requested capability is on the kernel's prohibited list.
    ProhibitedCapability,
    /// The `max_rate_hz` exceeds the policy maximum.
    RateTooHigh,
    /// The manifest is malformed — missing required field or invalid type.
    Malformed,
    /// The app_id is already registered.
    DuplicateAppId,
}

/// Specific wire-format protocol errors.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProtocolFault {
    /// The frame header was truncated.
    TruncatedHeader,
    /// The frame body was truncated.
    TruncatedBody,
    /// An unknown frame type was received.
    UnknownFrameType(u16),
    /// A required field was missing.
    MissingField(&'static str),
    /// A field had an unexpected type.
    InvalidFieldType(&'static str),
    /// The frame size exceeded the maximum allowed.
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
        assert!(!Error::TransportUnreachable(TransportFault::Timeout).is_terminal());
    }

    #[test]
    fn error_codes_are_stable() {
        // These values are part of the public ABI. Changing them is a
        // breaking change. This test documents the current values.
        assert_eq!(Error::ConsentWithdrawn.code() as u16, 0x0302);
        assert_eq!(Error::ConsentSuspended.code() as u16, 0x0301);
        assert_eq!(Error::AttestationFailed.code() as u16, 0x0402);
    }

    #[test]
    fn display_works_in_no_std_path() {
        // We test the Display impl works — exact formatting is not part of
        // the stable contract.
        let err = Error::RateLimitExceeded { max_rate_hz: 50 };
        // Using format! requires alloc, so we smoke-test via Debug.
        let _ = format!("{err:?}");
        let _ = format!("{err}");
    }
}
