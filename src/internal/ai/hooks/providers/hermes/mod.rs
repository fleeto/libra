//! Local CLI scope admission for Hermes.
//!
//! HM-09 exposes the pure parser and scope API. It does not register a
//! [`HookProvider`](crate::internal::ai::hooks::provider::HookProvider).
//! HM-05 owns observer installation.

pub mod parser;
pub mod scope;

pub use parser::parse_hermes_scope_value;
pub use scope::{
    HermesScopeCandidate, HermesScopeDecision, HermesVerifiedWorktree, admit_hermes_scope,
    admit_producer, admit_receiver,
};
