// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou / AxonOS

//! Application manifest — the declaration every AxonOS application sends
//! to the kernel at handshake.
//!
//! # Fields
//!
//! The manifest is **signed by the application publisher** (in production
//! builds). The kernel verifies the signature against a locally-installed
//! trust root before allowing the application to subscribe to any intent
//! stream. This SDK does not implement signing; that is done by an
//! out-of-band build step and the signature blob is attached at runtime.
//!
//! # Validation
//!
//! [`ManifestBuilder::build`] performs local validation:
//! - `app_id` is non-empty and ≤ 64 UTF-8 bytes (mirrors AxonOS kernel limits)
//! - At least one capability is declared
//! - `max_rate_hz` does not exceed the kernel rate limit for any declared
//!   capability, and is strictly positive.
//!
//! Kernel-side validation (signature verification, policy checks) happens
//! only at handshake time and returns [`crate::Error::ManifestRejected`].

use crate::capability::{Capability, CapabilitySet};
use crate::error::{Error, ManifestRejection, Result};
use heapless::String;

/// Maximum length of an app_id string, in UTF-8 bytes.
pub const MAX_APP_ID_LEN: usize = 64;

/// Maximum length of display name / vendor strings.
pub const MAX_DISPLAY_STRING_LEN: usize = 64;

/// A signed declaration of what an AxonOS application is authorized to do.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Manifest {
    /// Reverse-DNS application identifier, e.g., `com.example.cursor`.
    app_id: String<MAX_APP_ID_LEN>,
    /// Declared capabilities.
    capabilities: CapabilitySet,
    /// Maximum event rate requested by the application, across all streams.
    max_rate_hz: u32,
    /// Optional human-readable application name for UI display. Not used
    /// for protocol decisions.
    name: Option<String<MAX_DISPLAY_STRING_LEN>>,
    /// Optional vendor / publisher string.
    vendor: Option<String<MAX_DISPLAY_STRING_LEN>>,
}

impl Manifest {
    /// Start building a manifest.
    #[must_use]
    pub fn builder() -> ManifestBuilder {
        ManifestBuilder::default()
    }

    /// Reverse-DNS app identifier.
    #[must_use]
    pub fn app_id(&self) -> &str {
        self.app_id.as_str()
    }

    /// Declared capability set.
    #[must_use]
    pub const fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }

    /// Maximum event rate, Hz.
    #[must_use]
    pub const fn max_rate_hz(&self) -> u32 {
        self.max_rate_hz
    }

    /// Human-readable application name, if set.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Vendor / publisher string, if set.
    #[must_use]
    pub fn vendor(&self) -> Option<&str> {
        self.vendor.as_deref()
    }

    /// Check whether this manifest declares a capability.
    #[must_use]
    pub const fn allows(&self, c: Capability) -> bool {
        self.capabilities.contains(c)
    }
}

/// Builder for [`Manifest`].
///
/// # Ergonomics
/// All intermediate builder methods are infallible. Validation is deferred
/// to [`ManifestBuilder::build`], allowing ergonomic chained construction:
///
/// ```
/// use axonos_sdk::{Manifest, Capability};
///
/// let manifest = Manifest::builder()
///     .app_id("com.example.cursor")
///     .capability(Capability::Navigation)
///     .max_rate_hz(50)
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Default, Clone)]
pub struct ManifestBuilder {
    app_id: Option<String<MAX_APP_ID_LEN>>,
    capabilities: CapabilitySet,
    max_rate_hz: Option<u32>,
    name: Option<String<MAX_DISPLAY_STRING_LEN>>,
    vendor: Option<String<MAX_DISPLAY_STRING_LEN>>,
}

impl ManifestBuilder {
    /// Set the app_id (reverse-DNS). Required.
    ///
    /// Validation (non-empty, ≤ 64 bytes) happens in [`build`].
    #[must_use]
    pub fn app_id(mut self, id: &str) -> Self {
        let mut s = String::new();
        // Best-effort storage; validation deferred to build().
        let _ = s.push_str(id);
        self.app_id = Some(s);
        self
    }

    /// Declare a capability. Can be called multiple times.
    #[must_use]
    pub fn capability(mut self, c: Capability) -> Self {
        self.capabilities = self.capabilities.with(c);
        self
    }

    /// Declare a maximum event rate (Hz). Must not exceed the kernel rate
    /// limit for any declared capability, and must be > 0.
    #[must_use]
    pub fn max_rate_hz(mut self, hz: u32) -> Self {
        self.max_rate_hz = Some(hz);
        self
    }

    /// Optional display name (≤ 64 UTF-8 bytes).
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        let mut s = String::new();
        let _ = s.push_str(name);
        self.name = Some(s);
        self
    }

    /// Optional vendor string (≤ 64 UTF-8 bytes).
    #[must_use]
    pub fn vendor(mut self, vendor: &str) -> Self {
        let mut s = String::new();
        let _ = s.push_str(vendor);
        self.vendor = Some(s);
        self
    }

    /// Finalize the manifest. Performs local validation.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ManifestRejected`] if:
    /// - `app_id` is missing or empty.
    /// - `app_id` exceeds 64 UTF-8 bytes.
    /// - No capabilities are declared.
    /// - `max_rate_hz` is 0.
    /// - `max_rate_hz` exceeds the kernel limit for any declared capability.
    pub fn build(self) -> Result<Manifest> {
        let app_id = self.app_id.ok_or(Error::ManifestRejected {
            reason: ManifestRejection::Malformed,
        })?;

        if app_id.is_empty() || app_id.len() > MAX_APP_ID_LEN {
            return Err(Error::ManifestRejected {
                reason: ManifestRejection::Malformed,
            });
        }

        if self.capabilities.is_empty() {
            return Err(Error::ManifestRejected {
                reason: ManifestRejection::Malformed,
            });
        }

        let max_rate_hz = self.max_rate_hz.unwrap_or(1);
        if max_rate_hz == 0 {
            return Err(Error::ManifestRejected {
                reason: ManifestRejection::Malformed,
            });
        }

        // Validate display strings do not exceed kernel limits.
        if let Some(ref n) = self.name {
            if n.len() > MAX_DISPLAY_STRING_LEN {
                return Err(Error::ManifestRejected {
                    reason: ManifestRejection::Malformed,
                });
            }
        }
        if let Some(ref v) = self.vendor {
            if v.len() > MAX_DISPLAY_STRING_LEN {
                return Err(Error::ManifestRejected {
                    reason: ManifestRejection::Malformed,
                });
            }
        }

        // Verify rate does not exceed kernel limit for any declared capability.
        for c in self.capabilities.iter() {
            if max_rate_hz > c.kernel_rate_limit_hz() {
                return Err(Error::ManifestRejected {
                    reason: ManifestRejection::RateTooHigh,
                });
            }
        }

        Ok(Manifest {
            app_id,
            capabilities: self.capabilities,
            max_rate_hz,
            name: self.name,
            vendor: self.vendor,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_valid_manifest() {
        let m = Manifest::builder()
            .app_id("com.example.a")
            .capability(Capability::Navigation)
            .max_rate_hz(10)
            .build()
            .unwrap();
        assert_eq!(m.app_id(), "com.example.a");
        assert!(m.allows(Capability::Navigation));
        assert!(!m.allows(Capability::WorkloadAdvisory));
    }

    #[test]
    fn empty_app_id_rejected() {
        let r = Manifest::builder().app_id("").capability(Capability::Navigation).build();
        assert!(r.is_err());
    }

    #[test]
    fn oversized_app_id_rejected() {
        let huge = "a".repeat(MAX_APP_ID_LEN + 1);
        let r = Manifest::builder()
            .app_id(&huge)
            .capability(Capability::Navigation)
            .build();
        assert!(r.is_err());
    }

    #[test]
    fn no_capabilities_rejected() {
        let r = Manifest::builder()
            .app_id("com.a")
            .max_rate_hz(1)
            .build();
        assert!(matches!(
            r,
            Err(Error::ManifestRejected {
                reason: ManifestRejection::Malformed
            })
        ));
    }

    #[test]
    fn zero_rate_rejected() {
        let r = Manifest::builder()
            .app_id("com.a")
            .capability(Capability::Navigation)
            .max_rate_hz(0)
            .build();
        assert!(matches!(
            r,
            Err(Error::ManifestRejected {
                reason: ManifestRejection::Malformed
            })
        ));
    }

    #[test]
    fn rate_exceeding_kernel_limit_rejected() {
        let r = Manifest::builder()
            .app_id("com.a")
            .capability(Capability::WorkloadAdvisory) // kernel limit = 1 Hz
            .max_rate_hz(10)
            .build();
        assert!(matches!(
            r,
            Err(Error::ManifestRejected {
                reason: ManifestRejection::RateTooHigh
            })
        ));
    }

    #[test]
    fn rate_within_kernel_limit_accepted() {
        let r = Manifest::builder()
            .app_id("com.a")
            .capability(Capability::Navigation) // kernel limit = 50 Hz
            .max_rate_hz(30)
            .build();
        assert!(r.is_ok());
    }

    #[test]
    fn builder_is_ergonomic() {
        // All intermediate steps are infallible.
        let m = Manifest::builder()
            .app_id("com.ergonomic.test")
            .name("Test App")
            .vendor("AxonOS")
            .capability(Capability::Navigation)
            .capability(Capability::SessionQuality)
            .max_rate_hz(25)
            .build()
            .unwrap();
        assert_eq!(m.name(), Some("Test App"));
        assert_eq!(m.vendor(), Some("AxonOS"));
    }
}
