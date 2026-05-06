// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou / AxonOS

//! Application manifest — declaration sent to kernel at handshake.

use crate::capability::{Capability, CapabilitySet};
use crate::error::{Error, ManifestRejection, Result};
use heapless::String;

/// Maximum app_id length in UTF-8 bytes.
pub const MAX_APP_ID_LEN: usize = 64;

/// Maximum display name / vendor length.
pub const MAX_DISPLAY_STRING_LEN: usize = 64;

/// Signed declaration of what an AxonOS application is authorized to do.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Manifest {
    app_id: String<MAX_APP_ID_LEN>,
    capabilities: CapabilitySet,
    max_rate_hz: u32,
    name: Option<String<MAX_DISPLAY_STRING_LEN>>,
    vendor: Option<String<MAX_DISPLAY_STRING_LEN>>,
}

impl Manifest {
    #[must_use]
    pub fn builder() -> ManifestBuilder {
        ManifestBuilder::default()
    }

    #[must_use]
    pub fn app_id(&self) -> &str {
        self.app_id.as_str()
    }

    #[must_use]
    pub const fn capabilities(&self) -> &CapabilitySet {
        &self.capabilities
    }

    #[must_use]
    pub const fn max_rate_hz(&self) -> u32 {
        self.max_rate_hz
    }

    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    #[must_use]
    pub fn vendor(&self) -> Option<&str> {
        self.vendor.as_deref()
    }

    #[must_use]
    pub const fn allows(&self, c: Capability) -> bool {
        self.capabilities.contains(c)
    }
}

/// Builder for [`Manifest`]. All intermediate methods are infallible.
#[derive(Debug, Default, Clone)]
pub struct ManifestBuilder {
    app_id: Option<String<MAX_APP_ID_LEN>>,
    capabilities: CapabilitySet,
    max_rate_hz: Option<u32>,
    name: Option<String<MAX_DISPLAY_STRING_LEN>>,
    vendor: Option<String<MAX_DISPLAY_STRING_LEN>>,
}

impl ManifestBuilder {
    /// Set app_id. Panics if > 64 bytes or empty.
    #[must_use]
    pub fn app_id(mut self, id: &str) -> Self {
        assert!(
            !id.is_empty() && id.len() <= MAX_APP_ID_LEN,
            "app_id must be non-empty and ≤ {} UTF-8 bytes, got {} bytes: {:?}",
            MAX_APP_ID_LEN, id.len(), id
        );
        let mut s = String::new();
        s.push_str(id).expect("length already checked by assert");
        self.app_id = Some(s);
        self
    }

    #[must_use]
    pub fn capability(mut self, c: Capability) -> Self {
        self.capabilities = self.capabilities.with(c);
        self
    }

    #[must_use]
    pub fn max_rate_hz(mut self, hz: u32) -> Self {
        self.max_rate_hz = Some(hz);
        self
    }

    /// Set display name. Panics if > 64 bytes.
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        assert!(
            name.len() <= MAX_DISPLAY_STRING_LEN,
            "name must be ≤ {} UTF-8 bytes, got {} bytes",
            MAX_DISPLAY_STRING_LEN, name.len()
        );
        let mut s = String::new();
        s.push_str(name).expect("length already checked");
        self.name = Some(s);
        self
    }

    /// Set vendor. Panics if > 64 bytes.
    #[must_use]
    pub fn vendor(mut self, vendor: &str) -> Self {
        assert!(
            vendor.len() <= MAX_DISPLAY_STRING_LEN,
            "vendor must be ≤ {} UTF-8 bytes, got {} bytes",
            MAX_DISPLAY_STRING_LEN, vendor.len()
        );
        let mut s = String::new();
        s.push_str(vendor).expect("length already checked");
        self.vendor = Some(s);
        self
    }

    /// Finalize. Performs local validation.
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
        if max_rate_hz == 0 {
            return Err(Error::ManifestRejected {
                reason: ManifestRejection::Malformed,
            });
        }

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
    }

    #[test]
    #[should_panic(expected = "app_id must be non-empty")]
    fn empty_app_id_panics() {
        let _ = Manifest::builder().app_id("");
    }

    #[test]
    #[should_panic(expected = "app_id must be non-empty")]
    fn oversized_app_id_panics() {
        let huge = "a".repeat(MAX_APP_ID_LEN + 1);
        let _ = Manifest::builder().app_id(&huge);
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
            .capability(Capability::WorkloadAdvisory)
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
    fn builder_is_ergonomic() {
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
