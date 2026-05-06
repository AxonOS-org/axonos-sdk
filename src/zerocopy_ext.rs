// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (c) 2026 Denis Yermakou / AxonOS

//! Audited unsafe module for zero-copy FFI operations.
//!
//! # Safety contract
//!
//! This is the **only** module in the crate where `unsafe` is allowed.
//! All unsafe blocks must be accompanied by:
//! - A `// SAFETY:` comment explaining the invariant.
//! - A corresponding unit test.
//! - Review by at least two maintainers.
//!
//! # Current status
//!
//! Empty placeholder. Will contain:
//! - `mmap`-based ring buffer access
//! - DMA buffer handling for Cortex-M
//! - FFI boundary casts for `IntentObservation`

#![allow(unsafe_code)]

/// Marker type — zero-copy extension not yet implemented.
pub struct ZeroCopyExt;
