//! Lienzo Engine is the Nodalix-owned core for Lienzo.
//!
//! `lienzo-gnome` still links the C++ document engine. This crate does not
//! store pixels, decode files, or draw. A subsystem moves here only after
//! the existing corpus still passes. Upstream commits are recorded in
//! [`provenance`] so a later sync can see what was observed. They are not
//! a crate dependency.

/// Observed upstream commits. Full 40-character Git SHAs.
pub mod provenance {
    /// `SethRobinson/Patchy` default branch on 2026-10-06.
    pub const PATCHY_OBSERVED: &str = "4816d450e6a94e954017629745c06044186b3dec";

    /// `storytold/photocraft` commit imported under `engine/photocraft`.
    /// This crate does not recompile that tree. Build it from that directory.
    pub const PHOTOCRAFT_OBSERVED: &str = "909efc0f6df0a684d270019384e355c850642e42";
}

#[cfg(test)]
mod tests {
    use super::provenance;

    #[test]
    fn observed_shas_are_full_git_ids() {
        assert_eq!(provenance::PATCHY_OBSERVED.len(), 40);
        assert_eq!(provenance::PHOTOCRAFT_OBSERVED.len(), 40);
        assert_ne!(provenance::PATCHY_OBSERVED, provenance::PHOTOCRAFT_OBSERVED);
    }
}
