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
//!   capability
//!
//! Kernel-side validation (signature verification, policy checks) happens
//! only at handshake time and returns [`crate::Error::ManifestRejected`].

use crate::capability::{Capability, CapabilitySet};
use crate::error::{Error, ManifestRejection, Result};
use heapless::String;

/// Maximum length of an app_id string, in UTF-8 bytes.
pub const MAX_APP_ID_LEN: usize = 64;

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
    name: Option<String<64>>,
    /// Optional vendor / publisher string.
    vendor: Option<String<64>>,
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
#[derive(Debug, Default, Clone)]
pub struct ManifestBuilder {
    app_id: Option<String<MAX_APP_ID_LEN>>,
    capabilities: CapabilitySet,
    max_rate_hz: Option<u32>,
    name: Option<String<64>>,
    vendor: Option<String<64>>,
}

impl ManifestBuilder {
    /// Set the app_id (reverse-DNS). Required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ManifestRejected`] if the id is empty or exceeds
    /// [`MAX_APP_ID_LEN`] UTF-8 bytes.
    pub fn app_id(mut self, id: &str) -> Result<Self> {
        if id.is_empty() || id.len() > MAX_APP_ID_LEN {
            return Err(Error::ManifestRejected {
                reason: ManifestRejection::Malformed,
            });
        }
        let mut s = String::new();
        s.push_str(id).map_err(|()| Error::ManifestRejected {
            reason: ManifestRejection::Malformed,
        })?;
        self.app_id = Some(s);
        Ok(self)
    }

    /// Declare a capability. Can be called multiple times.
    #[must_use]
    pub fn capability(mut self, c: Capability) -> Self {
        self.capabilities = self.capabilities.with(c);
        self
    }

    /// Declare a maximum event rate (Hz). Must not exceed the kernel rate
    /// limit for any declared capability.
    #[must_use]
    pub const fn max_rate_hz(mut self, hz: u32) -> Self {
        self.max_rate_hz = Some(hz);
        self
    }

    /// Optional display name.
    pub fn name(mut self, name: &str) -> Result<Self> {
        if name.len() > 64 {
            return Err(Error::ManifestRejected {
                reason: ManifestRejection::Malformed,
            });
        }
        let mut s = String::new();
        s.push_str(name).map_err(|()| Error::ManifestRejected {
            reason: ManifestRejection::Malformed,
        })?;
        self.name = Some(s);
        Ok(self)
    }

    /// Optional vendor string.
    pub fn vendor(mut self, vendor: &str) -> Result<Self> {
        if vendor.len() > 64 {
            return Err(Error::ManifestRejected {
                reason: ManifestRejection::Malformed,
            });
        }
        let mut s = String::new();
        s.push_str(vendor).map_err(|()| Error::ManifestRejected {
            reason: ManifestRejection::Malformed,
        })?;
        self.vendor = Some(s);
        Ok(self)
    }

    /// Finalize the manifest. Performs local validation.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ManifestRejected`] if:
    /// - `app_id` is missing.
    /// - No capabilities are declared.
    /// - `max_rate_hz` exceeds the kernel limit for any declared capability.
    pub fn build(self) -> Result<Manifest> {
        let app_id = self.app_id.ok_or(Error::ManifestRejected {
            reason: ManifestRejection::Malformed,
        })?;

        if self.capabilities.is_empty() {
            return Err(Error::ManifestRejected {
                reason: ManifestRejection::Malformed,
            });
        }

        let max_rate_hz = self.max_rate_hz.unwrap_or(1);

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
            .unwrap()
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
        let r = Manifest::builder().app_id("");
        assert!(r.is_err());
    }

    #[test]
    fn oversized_app_id_rejected() {
        let huge = "a".repeat(MAX_APP_ID_LEN + 1);
        let r = Manifest::builder().app_id(&huge);
        assert!(r.is_err());
    }

    #[test]
    fn no_capabilities_rejected() {
        let r = Manifest::builder()
            .app_id("com.a")
            .unwrap()
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
    fn rate_exceeding_kernel_limit_rejected() {
        let r = Manifest::builder()
            .app_id("com.a")
            .unwrap()
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
            .unwrap()
            .capability(Capability::Navigation) // kernel limit = 50 Hz
            .max_rate_hz(30)
            .build();
        assert!(r.is_ok());
    }
}
