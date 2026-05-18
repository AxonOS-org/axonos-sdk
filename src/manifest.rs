// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou <denis@axonos.org>
// Part of the AxonOS project — https://github.com/AxonOS-org

//! Application manifest — declaration sent to kernel at handshake.
//!
//! # Builder pattern
//!
//! The builder is fallible at every setter. This is deliberate: input
//! validation must produce errors, not panics, especially in a
//! safety-oriented BCI context where panic = abort can terminate the
//! entire application.
//!
//! ```no_run
//! use axonos_sdk::{Manifest, Capability};
//!
//! let m = Manifest::builder()
//!     .app_id("com.example.app")?
//!     .capability(Capability::Navigation)
//!     .max_rate_hz(10)
//!     .build()?;
//! # Ok::<(), axonos_sdk::Error>(())
//! ```

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

/// Fallible builder for [`Manifest`].
///
/// All setters return `Result<Self, Error>` because input validation
/// must produce errors, not panics, in a safety-oriented context.
#[derive(Debug, Default, Clone)]
pub struct ManifestBuilder {
    app_id: Option<String<MAX_APP_ID_LEN>>,
    capabilities: CapabilitySet,
    max_rate_hz: Option<u32>,
    name: Option<String<MAX_DISPLAY_STRING_LEN>>,
    vendor: Option<String<MAX_DISPLAY_STRING_LEN>>,
}

impl ManifestBuilder {
    /// Set app_id. Returns error if empty or exceeds [`MAX_APP_ID_LEN`].
    pub fn app_id(mut self, id: &str) -> Result<Self> {
        if id.is_empty() || id.len() > MAX_APP_ID_LEN {
            return Err(Error::ManifestRejected {
                reason: ManifestRejection::Malformed,
            });
        }
        let mut s = String::new();
        // Cannot fail: length already validated.
        s.push_str(id).map_err(|_| Error::ManifestRejected {
            reason: ManifestRejection::Malformed,
        })?;
        self.app_id = Some(s);
        Ok(self)
    }

    /// Add a capability. Infallible — capability is an enum.
    #[must_use]
    pub fn capability(mut self, c: Capability) -> Self {
        self.capabilities = self.capabilities.with(c);
        self
    }

    /// Set the global rate ceiling.
    ///
    /// Per-capability kernel limits still apply: the effective rate for
    /// capability `c` is `min(max_rate_hz, c.kernel_rate_limit_hz())`.
    /// See [`Capability::kernel_rate_limit_hz`].
    #[must_use]
    pub fn max_rate_hz(mut self, hz: u32) -> Self {
        self.max_rate_hz = Some(hz);
        self
    }

    /// Set display name. Returns error if exceeds [`MAX_DISPLAY_STRING_LEN`].
    pub fn name(mut self, name: &str) -> Result<Self> {
        if name.len() > MAX_DISPLAY_STRING_LEN {
            return Err(Error::ManifestRejected {
                reason: ManifestRejection::Malformed,
            });
        }
        let mut s = String::new();
        s.push_str(name).map_err(|_| Error::ManifestRejected {
            reason: ManifestRejection::Malformed,
        })?;
        self.name = Some(s);
        Ok(self)
    }

    /// Set vendor. Returns error if exceeds [`MAX_DISPLAY_STRING_LEN`].
    pub fn vendor(mut self, vendor: &str) -> Result<Self> {
        if vendor.len() > MAX_DISPLAY_STRING_LEN {
            return Err(Error::ManifestRejected {
                reason: ManifestRejection::Malformed,
            });
        }
        let mut s = String::new();
        s.push_str(vendor).map_err(|_| Error::ManifestRejected {
            reason: ManifestRejection::Malformed,
        })?;
        self.vendor = Some(s);
        Ok(self)
    }

    /// Finalize the manifest. Performs all cross-field validation.
    ///
    /// # Errors
    /// - [`ManifestRejection::Malformed`] — missing app_id, no capabilities,
    ///   or zero `max_rate_hz`.
    /// - [`ManifestRejection::RateTooHigh`] — `max_rate_hz` exceeds the
    ///   kernel's per-capability limit. The check uses the minimum
    ///   capability limit across the declared set, so applications with
    ///   mixed capabilities should request the lowest needed rate.
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

        // Cross-field validation: max_rate_hz must not exceed the lowest
        // per-capability limit in the declared set.
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
    }

    #[test]
    fn empty_app_id_returns_error() {
        let r = Manifest::builder().app_id("");
        assert!(matches!(
            r,
            Err(Error::ManifestRejected {
                reason: ManifestRejection::Malformed
            })
        ));
    }

    #[test]
    fn oversized_app_id_returns_error() {
        let huge = "a".repeat(MAX_APP_ID_LEN + 1);
        let r = Manifest::builder().app_id(&huge);
        assert!(matches!(
            r,
            Err(Error::ManifestRejected {
                reason: ManifestRejection::Malformed
            })
        ));
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
    fn zero_rate_rejected() {
        let r = Manifest::builder()
            .app_id("com.a")
            .unwrap()
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
        // WorkloadAdvisory cap is 1 Hz, request 10 Hz → reject
        let r = Manifest::builder()
            .app_id("com.a")
            .unwrap()
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
    fn full_builder_chain() {
        let m = Manifest::builder()
            .app_id("com.ergonomic.test")
            .unwrap()
            .name("Test App")
            .unwrap()
            .vendor("AxonOS")
            .unwrap()
            .capability(Capability::Navigation)
            .capability(Capability::SessionQuality)
            .max_rate_hz(2) // limited by SessionQuality (2 Hz)
            .build()
            .unwrap();
        assert_eq!(m.name(), Some("Test App"));
        assert_eq!(m.vendor(), Some("AxonOS"));
        assert_eq!(m.max_rate_hz(), 2);
    }

    #[test]
    fn no_panic_on_malicious_input() {
        // Property: no input through public API can cause a panic.
        let inputs = ["", "x", &"a".repeat(64), &"a".repeat(65), &"a".repeat(1024)];
        for s in &inputs {
            let _ = Manifest::builder().app_id(s);
            let _ = Manifest::builder().name(s);
            let _ = Manifest::builder().vendor(s);
        }
    }
}

