//! Version guard: a binary older than the harness requires refuses to write harness files.

use anyhow::{Context, Result, bail};
use semver::Version;

pub const BINARY_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Fails when this binary is older than `minimum`, naming the required version and who requires it.
pub fn require(minimum: &str, required_by: &str) -> Result<()> {
    let wanted =
        parse(minimum).with_context(|| format!("{required_by}: minimum workbench version"))?;
    let current = Version::parse(BINARY_VERSION).expect("the crate version is semver");
    if current < wanted {
        bail!(
            "{required_by} requires workbench {wanted} or newer, but this is workbench {current}; \
             upgrade workbench to {wanted} and run the command again (nothing was written)"
        );
    }
    Ok(())
}

/// Parses a version, accepting a leading `v` as release tags carry.
pub fn parse(text: &str) -> Result<Version> {
    let trimmed = text.trim();
    let bare = trimmed.strip_prefix('v').unwrap_or(trimmed);
    Version::parse(bare).with_context(|| format!("`{text}` is not a semantic version"))
}
