//! Observe-only Grok hook provider.
//!
//! The CLI enable gate stays closed until GK-05. Tests call
//! [`install_marked_grok_hooks`] and [`observe_grok_hook`] directly.
//! Nothing in this module starts a process.

pub mod parser;
pub mod settings;

use anyhow::{Result, bail};
pub use parser::{encode_grok_cwd, observe_grok_hook};
pub use settings::{install_marked_grok_hooks, uninstall_marked_grok_hooks};

use super::super::{
    lifecycle::{LifecycleEvent, SessionHookEnvelope},
    provider::{
        CANONICAL_DEDUP_IDENTITY_KEYS, HookProvider, ProviderHookCommand, ProviderInstallOptions,
    },
};

pub static GROK_PROVIDER: GrokProvider = GrokProvider;

const SUPPORTED_COMMANDS: &[ProviderHookCommand] = &[
    ProviderHookCommand::Stop,
    ProviderHookCommand::Compaction,
    ProviderHookCommand::SessionEnd,
];

#[derive(Debug, Clone, Copy)]
pub struct GrokProvider;

impl HookProvider for GrokProvider {
    fn provider_name(&self) -> &'static str {
        "grok"
    }

    fn source_name(&self) -> &'static str {
        "grok_hook"
    }

    fn supported_commands(&self) -> &'static [ProviderHookCommand] {
        SUPPORTED_COMMANDS
    }

    fn parse_hook_event(
        &self,
        hook_event_name: &str,
        envelope: &SessionHookEnvelope,
    ) -> Result<LifecycleEvent> {
        parser::parse_grok_hook_event(hook_event_name, envelope)
    }

    fn recognizes_event(&self, hook_event_name: &str) -> bool {
        parser::recognizes_grok_event(hook_event_name)
    }

    fn dedup_identity_keys(&self) -> &'static [&'static str] {
        CANONICAL_DEDUP_IDENTITY_KEYS
    }

    fn install_hooks(&self, _options: &ProviderInstallOptions) -> Result<()> {
        bail!(
            "grok hook installation stays closed until hook_installable is enabled; call install_marked_grok_hooks with an explicit directory"
        )
    }

    fn uninstall_hooks(&self) -> Result<()> {
        bail!(
            "grok hook removal stays closed until hook_installable is enabled; call uninstall_marked_grok_hooks with an explicit directory"
        )
    }

    fn hooks_are_installed(&self) -> Result<bool> {
        Ok(false)
    }
}
