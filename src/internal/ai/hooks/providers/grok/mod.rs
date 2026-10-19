//! Observe-only Grok hook provider.
//!
//! `parse_hook_event` records the local GK-04 capture for Stop,
//! StopCancelled, StopFailure, PreCompact, and SessionEnd. It does not
//! export through the OpenCode bridge and does not start `grok`.

pub mod parser;
pub mod settings;

use anyhow::{Result, bail};
pub use parser::{encode_grok_cwd, observe_grok_hook};
pub use settings::{
    grok_hook_command, grok_user_hooks_dir, install_grok_boundary_hooks, install_marked_grok_hooks,
    marked_hooks_present, uninstall_marked_grok_hooks,
};

use super::super::{
    lifecycle::{LifecycleEvent, SessionHookEnvelope},
    provider::{
        CANONICAL_DEDUP_IDENTITY_KEYS, HookProvider, ProviderHookCommand, ProviderInstallOptions,
    },
};

pub static GROK_PROVIDER: GrokProvider = GrokProvider;

fn record_grok_boundary(envelope: &SessionHookEnvelope) {
    let Some(hooks_dir) = settings::grok_user_hooks_dir() else {
        return;
    };
    let Some(home) = hooks_dir.parent() else {
        return;
    };
    let mut event = serde_json::json!({
        "hook_event_name": envelope.hook_event_name,
        "sessionId": envelope.session_id,
        "cwd": envelope.cwd,
    });
    if let Some(path) = &envelope.transcript_path {
        event["transcript_path"] = serde_json::Value::String(path.clone());
    }
    parser::observe_grok_hook(
        &home.join("sessions"),
        &home.join("libra-capture"),
        &event,
        None,
    );
}

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
        let event = parser::parse_grok_hook_event(hook_event_name, envelope)?;
        record_grok_boundary(envelope);
        Ok(event)
    }

    fn recognizes_event(&self, hook_event_name: &str) -> bool {
        parser::recognizes_grok_event(hook_event_name)
    }

    fn dedup_identity_keys(&self) -> &'static [&'static str] {
        CANONICAL_DEDUP_IDENTITY_KEYS
    }

    fn install_hooks(&self, options: &ProviderInstallOptions) -> Result<()> {
        let binary = super::super::setup::resolve_hook_binary_path(options.binary_path.as_deref())?;
        let Some(dir) = settings::grok_user_hooks_dir() else {
            bail!("GROK_HOME is unset and the user home directory is unavailable");
        };
        settings::install_grok_boundary_hooks(&dir, &binary)?;
        Ok(())
    }

    fn uninstall_hooks(&self) -> Result<()> {
        let Some(dir) = settings::grok_user_hooks_dir() else {
            return Ok(());
        };
        settings::uninstall_marked_grok_hooks(&dir)?;
        Ok(())
    }

    fn hooks_are_installed(&self) -> Result<bool> {
        Ok(settings::grok_user_hooks_dir()
            .as_deref()
            .is_some_and(settings::marked_hooks_present))
    }
}
