//! Local CLI scope admission and the observe-only Hermes hook.
//!
//! HM-09 exposes the pure parser and scope API. HM-05 accepts the proven
//! observers through `libra agent hooks hermes observe`. HM-06 installs the
//! Libra-owned plugin when the marker and sha256 match. Review and
//! investigate launch stay closed.

pub mod parser;
pub mod scope;
pub mod settings;

use anyhow::Result;
pub use parser::parse_hermes_scope_value;
pub use scope::{
    HermesScopeCandidate, HermesScopeDecision, HermesVerifiedWorktree, admit_hermes_scope,
    admit_producer, admit_receiver,
};

use super::super::{
    lifecycle::{LifecycleEvent, SessionHookEnvelope},
    provider::{HookProvider, ProviderHookCommand, ProviderInstallOptions},
};
use crate::internal::ai::observed_agents::AgentKind;

pub static HERMES_PROVIDER: HermesProvider = HermesProvider;

const SUPPORTED_COMMANDS: &[ProviderHookCommand] = &[ProviderHookCommand::Observe];

const DEDUP_KEYS: &[&str] = &["event_id", "turn_id", "message_id", "phase"];

#[derive(Debug, Clone, Copy)]
pub struct HermesProvider;

impl crate::internal::ai::hooks::provider::HookProviderIdentity for HermesProvider {
    fn agent_kind(&self) -> AgentKind {
        AgentKind::Hermes
    }
}

impl HookProvider for HermesProvider {
    fn provider_name(&self) -> &'static str {
        "hermes"
    }

    fn source_name(&self) -> &'static str {
        "hermes_observer"
    }

    fn supported_commands(&self) -> &'static [ProviderHookCommand] {
        SUPPORTED_COMMANDS
    }

    fn parse_hook_event(
        &self,
        hook_event_name: &str,
        envelope: &SessionHookEnvelope,
    ) -> Result<LifecycleEvent> {
        parser::parse_hermes_hook_event(hook_event_name, envelope)
    }

    fn recognizes_event(&self, hook_event_name: &str) -> bool {
        parser::recognizes_hermes_event(hook_event_name)
    }

    fn admit_ingress_frame(&self, hook_event_name: &str, _session_id: &str) -> bool {
        parser::hermes_event_is_ingested(hook_event_name)
    }

    fn dedup_identity_keys(&self) -> &'static [&'static str] {
        DEDUP_KEYS
    }

    fn command_output(&self, _command: ProviderHookCommand) -> Option<serde_json::Value> {
        None
    }

    fn install_hooks(&self, options: &ProviderInstallOptions) -> Result<()> {
        settings::install_hermes_hooks(options)
    }

    fn uninstall_hooks(&self) -> Result<()> {
        settings::uninstall_hermes_hooks()
    }

    fn hooks_are_installed(&self) -> Result<bool> {
        settings::hermes_hooks_are_installed()
    }
}

pub fn hermes_provider() -> &'static dyn HookProvider {
    &HERMES_PROVIDER
}
