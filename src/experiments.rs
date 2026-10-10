//! Candidate implementations under evaluation, enabled by the `experiments`
//! feature. Not part of the public API, and empty on the main branch.
//!
//! Add a candidate as a `#[inline] pub const fn` next to the function it would
//! replace, with a non-inlined wrapper in the `asm` module below, so that it
//! compiles the same way as the real function and its assembly can be dumped
//! with `scripts/asm_mca.sh`. Then add it to `benches/exp.rs` and to
//! `tests/exhaustive.rs`. See CONTRIBUTING.md for the workflow.

/// Non-inlined wrappers of the candidates for dumping their assembly.
#[cfg(feature = "asmdump")]
pub mod asm {}
