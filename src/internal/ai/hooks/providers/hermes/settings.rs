//! Hermes hook installation stays closed until capture is publicly supported.

use anyhow::{Result, bail};

use super::super::super::provider::ProviderInstallOptions;

pub fn install_hermes_hooks(_options: &ProviderInstallOptions) -> Result<()> {
    bail!("hermes hook installation stays disabled");
}

pub fn uninstall_hermes_hooks() -> Result<()> {
    Ok(())
}

pub fn hermes_hooks_are_installed() -> Result<bool> {
    Ok(false)
}
