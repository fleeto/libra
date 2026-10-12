//! Provider-neutral reasoning states and an opaque ciphertext type.
//!
//! This module validates a declared Claude field shape after source
//! authorization. Provider adapters must establish identity from pinned source
//! evidence; entropy and base64-like contents never prove ciphertext.

use std::{collections::BTreeSet, fmt};

use serde::{
    Deserialize, Serialize,
    de::{self, Deserializer, MapAccess, SeqAccess, Visitor},
};
use serde_json::value::RawValue;
use sha2::{Digest, Sha256};
use thiserror::Error;

use super::transcript_source::TRANSCRIPT_READ_HARD_CAP_BYTES;

/// Optional provider port at the authorized, bounded source boundary.
/// Implementations only separate an owned buffer. They cannot open a source,
/// access a store, or construct native authority from a provider label.
pub trait ReasoningSourcePreprocessor: Send + Sync {
    fn prepare_authorized_source(
        &self,
        source: Vec<u8>,
    ) -> Result<PreparedReasoningSource, ReasoningFieldError>;
}

/// A separated ordinary buffer and a sealed, owning disposition. No raw
/// accessor, serialization, cloning or content-bearing Debug is available.
pub struct PreparedReasoningSource {
    ordinary: Vec<u8>,
    disposition: ReasoningSourceDisposition,
}

impl PreparedReasoningSource {
    /// The selected, reviewed detector found no reasoning candidate.
    pub fn without_candidates(ordinary: Vec<u8>) -> Self {
        Self {
            ordinary,
            disposition: ReasoningSourceDisposition::Ordinary,
        }
    }

    /// The detector removed candidates whose native provenance is absent.
    /// The safe result remains nonordinary even when it has no artifact.
    pub fn unproven_candidates(ordinary: Vec<u8>) -> Self {
        Self {
            ordinary,
            disposition: ReasoningSourceDisposition::Unproven,
        }
    }
}

/// Payload-free state readable by the narrow capture adapters. This is an
/// internal state, not a new public metadata enum or a native proof.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReasoningSourceState {
    Ordinary,
    Unproven,
    Unmapped,
    HelperUnsupported,
    NativeProven,
}

/// Owned reasoning authority travels with the source, never through a DTO.
/// Even a zero-artifact partial must not become ordinary complete content.
pub(crate) enum ReasoningSourceDisposition {
    Ordinary,
    Unproven,
    Unmapped,
    HelperUnsupported,
    #[allow(dead_code)] // OG-05 alone may admit a production native issuer.
    NativeProven(NativeProvenReasoning),
}

impl ReasoningSourceDisposition {
    pub(crate) fn from_prepared_authorized_source(
        prepared: PreparedReasoningSource,
        has_verified_source_mapping: bool,
        max_source_bytes: u64,
        provider: &str,
        capture_session: &str,
        source_session: Option<&str>,
    ) -> Result<(Vec<u8>, Self), ReasoningFieldError> {
        if prepared.ordinary.len() as u64 > max_source_bytes {
            return Err(ReasoningFieldError::SourceTooLarge);
        }
        let mut disposition = prepared.disposition;
        if !has_verified_source_mapping && !disposition.is_ordinary() {
            disposition = Self::Unmapped;
        } else if let Self::NativeProven(batch) = &disposition {
            let kind = match provider {
                "claude_code" => ReasoningProvider::ClaudeCode,
                "codex" => ReasoningProvider::Codex,
                _ => return Err(ReasoningFieldError::UnsupportedShape),
            };
            batch.validate(capture_session, kind)?;
            if source_session != Some(batch.source_session.as_str()) {
                return Err(ReasoningFieldError::UnsupportedShape);
            }
        }
        Ok((prepared.ordinary, disposition))
    }

    pub(crate) fn helper_unsupported() -> Self {
        Self::HelperUnsupported
    }

    pub(crate) fn is_ordinary(&self) -> bool {
        matches!(self, Self::Ordinary)
    }

    /// Whether the sealed source state has a downstream capture consumer.
    /// `NativeProven` is admitted only through the typed identity/claim path;
    /// all other nonordinary states remain fail-closed.
    pub(crate) fn has_capture_consumer(&self) -> bool {
        matches!(self, Self::Ordinary | Self::NativeProven(_))
    }

    pub(crate) fn state(&self) -> ReasoningSourceState {
        match self {
            Self::Ordinary => ReasoningSourceState::Ordinary,
            Self::Unproven => ReasoningSourceState::Unproven,
            Self::Unmapped => ReasoningSourceState::Unmapped,
            Self::HelperUnsupported => ReasoningSourceState::HelperUnsupported,
            Self::NativeProven(_) => ReasoningSourceState::NativeProven,
        }
    }
}

/// No production native issuer is admitted by this preparatory carrier card.
/// OG-05 may add the independently verified OpenCode issuer after native GO.
/// An opaque RG classification never constructs this sealed witness.
#[allow(dead_code)] // Admission remains closed until OG-05 native GO.
enum NativeReasoningIssuer {
    Unregistered,
    #[cfg(test)]
    SyntheticOnly,
    #[cfg(test)]
    SyntheticUnknown,
}

pub(crate) struct NativeReasoningProof {
    issuer: NativeReasoningIssuer,
    provider: ReasoningProvider,
    source_session: String,
    message: u64,
    part: u64,
    field: ReasoningSourceKind,
    original_wire_sha256: [u8; 32],
    logical_turn: String,
}

/// Non-Clone batch: artifact bytes and their witnesses have exactly one owner.
/// Its constructor remains inside this module's fixed source validator.
pub(crate) struct NativeProvenReasoning {
    capture_session: String,
    source_session: String,
    artifacts: Vec<crate::internal::ai::traces::ReasoningArtifactParam>,
    proofs: Vec<NativeReasoningProof>,
}

/// A capture association validated by the shared checkpoint adapter. It does
/// not mint a native witness and cannot be serialized into an IPC shortcut.
pub(crate) struct VerifiedCaptureBinding {
    capture_session: String,
    source_session: String,
    provider: ReasoningProvider,
}

impl VerifiedCaptureBinding {
    pub(crate) fn from_validated_capture_context(
        batch: &NativeProvenReasoning,
        capture_session: &str,
        provider: &str,
    ) -> Result<Self, ReasoningFieldError> {
        let provider = match provider {
            "claude_code" => ReasoningProvider::ClaudeCode,
            "codex" => ReasoningProvider::Codex,
            _ => return Err(ReasoningFieldError::UnsupportedShape),
        };
        batch.validate(capture_session, provider)?;
        Ok(Self {
            capture_session: capture_session.to_owned(),
            source_session: batch.source_session.clone(),
            provider,
        })
    }
}

/// Metadata-only borrowed identity inputs. The native bytes are not present
/// in this view, and no constructor or mutable accessor is exposed.
pub(crate) struct BoundIdentityInputRef<'a> {
    original_wire_sha256: &'a [u8; 32],
    logical_turn: &'a str,
    source_binding_sha256: [u8; 32],
    issuer_tag: &'static str,
}

#[allow(dead_code)] // OG-19/20 consume this metadata-only preparatory view.
impl<'a> BoundIdentityInputRef<'a> {
    fn from_bound_item(proof: &'a NativeReasoningProof) -> Self {
        let mut source_binding = Sha256::new();
        source_binding.update(b"libra/reasoning-source-binding/v1\0");
        for field in [
            b"claude_code".as_slice(),
            proof.source_session.as_bytes(),
            &proof.message.to_le_bytes(),
            &proof.part.to_le_bytes(),
            match proof.field {
                ReasoningSourceKind::Signature => b"signature".as_slice(),
                ReasoningSourceKind::RedactedThinkingData => b"data".as_slice(),
                _ => b"unsupported".as_slice(),
            },
        ] {
            source_binding.update((field.len() as u64).to_le_bytes());
            source_binding.update(field);
        }
        Self {
            original_wire_sha256: &proof.original_wire_sha256,
            logical_turn: &proof.logical_turn,
            source_binding_sha256: source_binding.finalize().into(),
            issuer_tag: match proof.issuer {
                NativeReasoningIssuer::Unregistered => "unregistered",
                #[cfg(test)]
                NativeReasoningIssuer::SyntheticOnly => "synthetic-test-only-v1",
                #[cfg(test)]
                NativeReasoningIssuer::SyntheticUnknown => "unknown-test-only",
            },
        }
    }

    pub(crate) fn original_wire_sha256(&self) -> &[u8; 32] {
        self.original_wire_sha256
    }

    pub(crate) fn logical_turn(&self) -> &str {
        self.logical_turn
    }

    pub(crate) fn source_binding_sha256(&self) -> &[u8; 32] {
        &self.source_binding_sha256
    }

    pub(crate) fn issuer_tag(&self) -> &'static str {
        self.issuer_tag
    }
}

pub(crate) fn borrow_native_identity_bindings<'a>(
    batch: &'a NativeProvenReasoning,
    binding: &VerifiedCaptureBinding,
) -> Result<Vec<BoundIdentityInputRef<'a>>, ReasoningFieldError> {
    batch.validate(&binding.capture_session, binding.provider)?;
    if batch.source_session != binding.source_session {
        return Err(ReasoningFieldError::UnsupportedShape);
    }
    Ok(batch
        .proofs
        .iter()
        .map(BoundIdentityInputRef::from_bound_item)
        .collect())
}

impl NativeProvenReasoning {
    fn validate(
        &self,
        capture_session: &str,
        provider: ReasoningProvider,
    ) -> Result<(), ReasoningFieldError> {
        use crate::internal::ai::traces::{
            REASONING_ARTIFACT_MAX_ENTRIES, REASONING_ARTIFACT_TOTAL_MAX_BYTES,
        };
        if capture_session.is_empty()
            || self.capture_session != capture_session
            || self.capture_session != format!("claude__{}", self.source_session)
            || self.source_session.is_empty()
            || self.artifacts.is_empty()
            || self.artifacts.len() != self.proofs.len()
            || self.artifacts.len() > REASONING_ARTIFACT_MAX_ENTRIES
        {
            return Err(ReasoningFieldError::UnsupportedShape);
        }
        let mut total = 0_u64;
        let mut locators = BTreeSet::new();
        for (artifact, proof) in self.artifacts.iter().zip(&self.proofs) {
            let issuer_registered = match proof.issuer {
                NativeReasoningIssuer::Unregistered => false,
                #[cfg(test)]
                NativeReasoningIssuer::SyntheticOnly => {
                    proof.logical_turn == format!("synthetic-turn-{}", proof.message)
                }
                #[cfg(test)]
                NativeReasoningIssuer::SyntheticUnknown => false,
            };
            let field = match proof.field {
                ReasoningSourceKind::Signature => "signature",
                ReasoningSourceKind::RedactedThinkingData => "data",
                _ => return Err(ReasoningFieldError::UnsupportedShape),
            };
            // OG-16's synthetic positive uses the already admitted Claude
            // locator. No production OpenCode locator or issuer is added.
            let locator = format!(
                "claude_code:msg={}/part={}/metadata={field}",
                proof.message, proof.part
            );
            if !issuer_registered
                || provider != ReasoningProvider::ClaudeCode
                || proof.provider != provider
                || artifact.provider != provider
                || proof.source_session != self.source_session
                || proof.logical_turn.is_empty()
                || artifact.source_kind != Some(proof.field)
                || artifact.bytes.verified_source() != (provider, proof.field)
                || artifact.locator != locator
                || !locators.insert(&artifact.locator)
                || !artifact_locator_is_valid(provider, Some(proof.field), &artifact.locator)
                || artifact.availability != ReasoningAvailability::OpaqueArchived
                || artifact.decrypt_capability != "none"
                || proof.original_wire_sha256
                    != <[u8; 32]>::from(Sha256::digest(artifact.bytes.as_bytes()))
            {
                return Err(ReasoningFieldError::UnsupportedShape);
            }
            total = total
                .checked_add(artifact.bytes.len() as u64)
                .ok_or(ReasoningFieldError::SourceTooLarge)?;
            if total > REASONING_ARTIFACT_TOTAL_MAX_BYTES {
                return Err(ReasoningFieldError::SourceTooLarge);
            }
        }
        Ok(())
    }

    pub(crate) fn artifact_params<'a>(
        &'a self,
        binding: &VerifiedCaptureBinding,
    ) -> Result<&'a [crate::internal::ai::traces::ReasoningArtifactParam], ReasoningFieldError>
    {
        self.validate(&binding.capture_session, binding.provider)?;
        if self.source_session != binding.source_session {
            return Err(ReasoningFieldError::UnsupportedShape);
        }
        Ok(&self.artifacts)
    }
}

#[cfg(test)]
pub(crate) fn synthetic_native_preparation(
    ordinary: Vec<u8>,
    capture_session: &str,
    source_session: &str,
    fields: Vec<Vec<u8>>,
) -> PreparedReasoningSource {
    let mut artifacts = Vec::with_capacity(fields.len());
    let mut proofs = Vec::with_capacity(fields.len());
    for (part, data) in fields.into_iter().enumerate() {
        let digest = Sha256::digest(&data).into();
        artifacts.push(crate::internal::ai::traces::ReasoningArtifactParam {
            locator: format!("claude_code:msg=0/part={part}/metadata=signature"),
            provider: ReasoningProvider::ClaudeCode,
            source_kind: Some(ReasoningSourceKind::Signature),
            availability: ReasoningAvailability::OpaqueArchived,
            decrypt_capability: "none".to_owned(),
            bytes: OpaqueEncryptedBytes {
                data,
                provider: ReasoningProvider::ClaudeCode,
                source_kind: ReasoningSourceKind::Signature,
            },
        });
        proofs.push(NativeReasoningProof {
            issuer: NativeReasoningIssuer::SyntheticOnly,
            provider: ReasoningProvider::ClaudeCode,
            source_session: source_session.to_owned(),
            message: 0,
            part: part as u64,
            field: ReasoningSourceKind::Signature,
            original_wire_sha256: digest,
            logical_turn: "synthetic-turn-0".to_owned(),
        });
    }
    PreparedReasoningSource {
        ordinary,
        disposition: ReasoningSourceDisposition::NativeProven(NativeProvenReasoning {
            capture_session: capture_session.to_owned(),
            source_session: source_session.to_owned(),
            artifacts,
            proofs,
        }),
    }
}

#[cfg(test)]
pub(crate) mod carrier_test_support {
    use super::*;
    use crate::internal::ai::observed_agents::{AgentKind, AgentSessionCtx, ObservedAgent};
    pub(crate) const CANARY: &[u8] = b"OG16_CANDIDATE_PRIVATE_CANARY";
    pub(crate) const SOURCE: &str = "og16-native-source";
    pub(crate) const CAPTURE: &str = "claude__og16-native-source";
    #[derive(Clone, Copy)]
    pub(crate) enum Mode {
        None,
        Strip,
        Native,
        Malformed,
        ArtifactOversize,
        TooMany,
        Unmapped,
        Helper,
    }
    pub(crate) struct Adapter(pub(crate) Mode);
    pub(crate) static NONE: Adapter = Adapter(Mode::None);
    pub(crate) static STRIP: Adapter = Adapter(Mode::Strip);
    pub(crate) static NATIVE: Adapter = Adapter(Mode::Native);
    pub(crate) static MALFORMED: Adapter = Adapter(Mode::Malformed);
    pub(crate) static OVERSIZE: Adapter = Adapter(Mode::ArtifactOversize);
    pub(crate) static TOO_MANY: Adapter = Adapter(Mode::TooMany);
    pub(crate) static UNMAPPED: Adapter = Adapter(Mode::Unmapped);
    pub(crate) static HELPER: Adapter = Adapter(Mode::Helper);
    impl ObservedAgent for Adapter {
        fn provider_kind(&self) -> AgentKind {
            AgentKind::ClaudeCode
        }
        fn provider_name(&self) -> &'static str {
            "claude-code"
        }
        fn read_transcript(&self, _: &AgentSessionCtx) -> anyhow::Result<Option<Vec<u8>>> {
            panic!("the source preparation port must not reopen a source")
        }
        fn protected_dirs(&self) -> &'static [&'static str] {
            &[".claude"]
        }
        fn as_reasoning_source_preprocessor(&self) -> Option<&dyn ReasoningSourcePreprocessor> {
            (!matches!(self.0, Mode::None)).then_some(self)
        }
    }
    impl ReasoningSourcePreprocessor for Adapter {
        fn prepare_authorized_source(
            &self,
            mut ordinary: Vec<u8>,
        ) -> Result<PreparedReasoningSource, ReasoningFieldError> {
            let candidate = ordinary.windows(CANARY.len()).any(|w| w == CANARY);
            while let Some(index) = ordinary.windows(CANARY.len()).position(|w| w == CANARY) {
                ordinary.drain(index..index + CANARY.len());
            }
            match self.0 {
                Mode::None => panic!("disabled accessor must not prepare"),
                Mode::Strip if candidate => {
                    Ok(PreparedReasoningSource::unproven_candidates(ordinary))
                }
                Mode::Strip => Ok(PreparedReasoningSource::without_candidates(ordinary)),
                Mode::Malformed => Err(ReasoningFieldError::UnsupportedShape),
                Mode::Unmapped => Ok(PreparedReasoningSource {
                    ordinary,
                    disposition: ReasoningSourceDisposition::Unmapped,
                }),
                Mode::Helper => Ok(PreparedReasoningSource {
                    ordinary,
                    disposition: ReasoningSourceDisposition::HelperUnsupported,
                }),
                Mode::Native => Ok(synthetic_native_preparation(
                    ordinary,
                    CAPTURE,
                    SOURCE,
                    vec![CANARY.to_vec(), b"second-opaque-wire".to_vec()],
                )),
                Mode::ArtifactOversize => Ok(synthetic_native_preparation(
                    ordinary,
                    CAPTURE,
                    SOURCE,
                    vec![vec![
                        b'x';
                        crate::internal::ai::traces::REASONING_ARTIFACT_TOTAL_MAX_BYTES
                            as usize
                            + 1
                    ]],
                )),
                Mode::TooMany => Ok(synthetic_native_preparation(
                    ordinary,
                    CAPTURE,
                    SOURCE,
                    vec![
                        vec![b'x'];
                        crate::internal::ai::traces::REASONING_ARTIFACT_MAX_ENTRIES + 1
                    ],
                )),
            }
        }
    }
    pub(crate) fn tamper_carrier_locator(disposition: &mut ReasoningSourceDisposition) {
        let ReasoningSourceDisposition::NativeProven(batch) = disposition else {
            panic!("synthetic native fixture")
        };
        batch.artifacts[0].locator = "claude_code:msg=0/part=100/metadata=signature".to_owned();
    }
}

/// Availability describes what the source proved, not whether capture failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningAvailability {
    ProviderVisible,
    EncryptedUnavailable,
    OpaqueArchived,
    NotPresent,
    UnsupportedShape,
}

/// Only registered provider tags can appear in safe reasoning metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningProvider {
    ClaudeCode,
    Codex,
    #[serde(rename = "opencode")]
    OpenCode,
}

/// A future provider's explicitly authorized, checkpoint-scoped capability.
/// Implementations must verify that their authorization covers checkpoint_id
/// and the ciphertext's provider. This registry currently grants none; callers
/// must retain encrypted_unavailable rather than infer a plaintext fallback.
/// No client key, config, environment or network discovery is performed here.
pub trait ReasoningDecryptor: Send + Sync {
    fn decrypt(
        &self,
        checkpoint_id: &str,
        ciphertext: &OpaqueEncryptedBytes,
    ) -> Result<ProviderVisibleText, ReasoningFieldError>;
}

/// No officially documented and authorized decryptor is implemented for any
/// observed provider. The capability is deliberately unavailable by default.
pub fn reasoning_decryptor(
    _provider: ReasoningProvider,
) -> Option<&'static dyn ReasoningDecryptor> {
    None
}

/// Metadata tags; no source text, paths, keys or original bytes are retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningSourceKind {
    Reasoning,
    Thinking,
    Summary,
    Signature,
    RedactedThinkingData,
    EncryptedContent,
}

/// Safe artifact locators contain only provider coordinates and a registered
/// source field. They are metadata, never a free-form provider string.
pub(crate) fn artifact_locator_is_valid(
    provider: ReasoningProvider,
    source_kind: Option<ReasoningSourceKind>,
    locator: &str,
) -> bool {
    fn ordinal(value: &str) -> bool {
        !value.is_empty() && value.len() <= 20 && value.bytes().all(|b| b.is_ascii_digit())
    }
    if locator.len() > 4096 {
        return false;
    }
    match (provider, source_kind) {
        (ReasoningProvider::ClaudeCode, Some(kind)) => {
            let field = match kind {
                ReasoningSourceKind::Signature => "signature",
                ReasoningSourceKind::RedactedThinkingData => "data",
                _ => return false,
            };
            let Some(rest) = locator.strip_prefix("claude_code:msg=") else {
                return false;
            };
            let Some((message, rest)) = rest.split_once("/part=") else {
                return false;
            };
            let Some((part, metadata)) = rest.split_once("/metadata=") else {
                return false;
            };
            ordinal(message) && ordinal(part) && metadata == field
        }
        (ReasoningProvider::Codex, Some(ReasoningSourceKind::EncryptedContent)) => {
            let Some(rest) = locator.strip_prefix("codex:ordinal=") else {
                return false;
            };
            if let Some((record, history)) = rest.split_once("/replacement_history[") {
                ordinal(record) && history.strip_suffix(']').is_some_and(ordinal)
            } else {
                ordinal(rest)
            }
        }
        // OpenCode has no source-proven opaque field at the current pin.
        _ => false,
    }
}

/// Diagnostic codes never contain a field path or source payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningWarning {
    UnsupportedShape,
}

/// A metadata-only classification. Persisting content is not part of RG-01.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct ReasoningRecord {
    provider: ReasoningProvider,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_kind: Option<ReasoningSourceKind>,
    availability: ReasoningAvailability,
    #[serde(skip_serializing_if = "Option::is_none")]
    warning: Option<ReasoningWarning>,
}

impl ReasoningRecord {
    /// The classification remains available to callers without exposing data.
    pub fn availability(&self) -> ReasoningAvailability {
        self.availability
    }

    pub fn source_kind(&self) -> Option<ReasoningSourceKind> {
        self.source_kind
    }

    /// Unknown reasoning-like structure produces a payload-free warning.
    pub fn warning(&self) -> Option<ReasoningWarning> {
        self.warning
    }
}

/// Only the provider adapter can establish whether a block is reasoning-like.
/// Unrecognized, high-entropy fields do not acquire that status by inspection.
pub enum UnrecognizedReasoningField {
    Other,
    ReasoningLike,
}

/// Classify an undeclared source field without examining or retaining its bytes.
/// Declared provider fields are handled only after source-shape verification.
pub fn classify_unrecognized_field(
    provider: ReasoningProvider,
    field: UnrecognizedReasoningField,
    _raw_value: &[u8],
) -> ReasoningRecord {
    let (availability, warning) = match field {
        UnrecognizedReasoningField::Other => (ReasoningAvailability::NotPresent, None),
        UnrecognizedReasoningField::ReasoningLike => (
            ReasoningAvailability::UnsupportedShape,
            Some(ReasoningWarning::UnsupportedShape),
        ),
    };
    ReasoningRecord {
        provider,
        source_kind: None,
        availability,
        warning,
    }
}

/// Classify a provider-supported, readable reasoning field: the record is
/// marked `ProviderVisible` and the text is returned behind the gated
/// [`ProviderVisibleText`] type so only the typed-redaction path can read it.
/// The source text is stored as-is (no decode/re-encode); redaction and
/// projection run before any persistence.
///
/// `pub(crate)` (Codex re-review P2): arbitrary external callers must not be
/// able to mint readable reasoning; adapters live in-crate and the type gate
/// (private `inner`, fail-closed `Serialize`) keeps the variant unforgeable
/// from outside the crate.
#[allow(dead_code)] // RG-04 adapters (claude/opencode classifier wiring) consume this entry.
pub(crate) fn classify_provider_visible_reasoning(
    provider: ReasoningProvider,
    source_kind: ReasoningSourceKind,
    text: String,
) -> (ReasoningRecord, ProviderVisibleText) {
    let record = ReasoningRecord {
        provider,
        source_kind: Some(source_kind),
        availability: ReasoningAvailability::ProviderVisible,
        warning: None,
    };
    let text = ProviderVisibleText::from_classification(&record, text);
    match text {
        Some(text) => (record, text),
        // INVARIANT: the record was constructed two lines above with
        // `availability: ReasoningAvailability::ProviderVisible`, so
        // `from_classification` cannot return None for it.
        None => unreachable!("record was just constructed with ProviderVisible availability"),
    }
}

/// Provider-visible reasoning text, constructible only through the
/// classification path that proved `ReasoningAvailability::ProviderVisible`.
///
/// The inner string is private and the type has no `From<String>`/`Display`
/// impl: external code cannot mint provider-visible reasoning by struct
/// literal or conversion (RG-04 compile_fail gate). Projection and redaction
/// read it via [`ProviderVisibleText::as_str`].
///
/// # Construction gate (RG-04 compile_fail)
///
/// ```compile_fail
/// use libra::internal::ai::observed_agents::coverage::SemanticRecord;
/// use libra::internal::ai::observed_agents::reasoning::{
///     ProviderVisibleText, ReasoningProvider, ReasoningSourceKind,
/// };
/// // The `inner` field is private and there is no public constructor or
/// // `From<String>` impl, so this struct literal cannot compile:
/// let text = ProviderVisibleText {
///     inner: "minted reasoning".to_string(),
/// };
/// let _ = SemanticRecord::Reasoning {
///     provider: ReasoningProvider::ClaudeCode,
///     source_kind: Some(ReasoningSourceKind::Reasoning),
///     text,
/// };
/// ```
#[derive(Clone, Eq, PartialEq)]
pub struct ProviderVisibleText {
    inner: String,
}

impl fmt::Debug for ProviderVisibleText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Codex re-review P1: `{:?}` must never leak the (possibly
        // unredacted) inner reasoning text into logs or diagnostics.
        f.debug_struct("ProviderVisibleText")
            .field("len", &self.inner.len())
            .field("inner", &"<redacted>".to_string())
            .finish()
    }
}

impl Serialize for ProviderVisibleText {
    fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
        // Codex re-review P1: serialization is deliberately unsupported. The
        // derived SemanticRecord::Serialize would otherwise provide a
        // pre-redaction persistence bypass; the sanctioned outputs are the
        // canonical writer and safe_turn_projection, which read `as_str()`
        // only after typed redaction has run.
        Err(serde::ser::Error::custom(
            "provider-visible reasoning text must be redacted first; use the canonical/projection writers",
        ))
    }
}

impl<'de> Deserialize<'de> for ProviderVisibleText {
    fn deserialize<D: Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        // Fail closed: provider-visible reasoning text can only be minted by
        // the classification path. Deserializing one from persisted JSON
        // would bypass the gate.
        Err(de::Error::custom(
            "provider-visible reasoning text cannot be deserialized; use the classification path",
        ))
    }
}

impl ProviderVisibleText {
    /// Constructible only from the classification path: the record must
    /// already carry `ReasoningAvailability::ProviderVisible`. Returns `None`
    /// when the availability is not `ProviderVisible` — the gate holds in
    /// release builds too (Codex re-review P1, not just a debug assert).
    #[allow(dead_code)] // RG-04 adapters consume this gated constructor.
    pub(crate) fn from_classification(record: &ReasoningRecord, text: String) -> Option<Self> {
        if record.availability() != ReasoningAvailability::ProviderVisible {
            return None;
        }
        Some(Self { inner: text })
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// In-crate trusted path for the typed-redaction step: redaction
    /// rewrites the text in place through the shared Redactor and returns its
    /// report. The inner string can only ever be redacted, never replaced
    /// with arbitrary text (Codex re-review P1).
    pub(crate) fn redact_with(
        &mut self,
        redactor: &super::redaction::Redactor,
    ) -> super::redaction::RedactionReport {
        let (bytes, report) = redactor.redact(self.inner.as_bytes());
        self.inner = String::from_utf8_lossy(bytes.as_ref()).into_owned();
        report
    }
}

/// Errors do not include source field names, values, or raw provider JSON.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum ReasoningFieldError {
    #[error("reasoning source exceeds the 16 MiB transcript read limit; use a bounded export")]
    SourceTooLarge,
    #[error("reasoning source is not a valid provider record; check the pinned provider export")]
    MalformedSource,
    #[error("reasoning field has an unsupported shape; update the pinned provider contract")]
    UnsupportedShape,
}

/// A verified in-memory source field, not an archived or decrypted artifact.
#[derive(Debug)]
pub struct VerifiedOpaqueReasoning {
    record: ReasoningRecord,
    bytes: OpaqueEncryptedBytes,
}

impl VerifiedOpaqueReasoning {
    pub fn record(&self) -> ReasoningRecord {
        self.record
    }

    pub fn into_opaque(self) -> OpaqueEncryptedBytes {
        self.bytes
    }
}

#[allow(dead_code)]
struct UniqueJsonKeys;

impl<'de> Deserialize<'de> for UniqueJsonKeys {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UniqueVisitor;

        impl<'de> Visitor<'de> for UniqueVisitor {
            type Value = UniqueJsonKeys;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON value with unique object keys")
            }

            fn visit_bool<E: de::Error>(self, _value: bool) -> Result<Self::Value, E> {
                Ok(UniqueJsonKeys)
            }

            fn visit_i64<E: de::Error>(self, _value: i64) -> Result<Self::Value, E> {
                Ok(UniqueJsonKeys)
            }

            fn visit_u64<E: de::Error>(self, _value: u64) -> Result<Self::Value, E> {
                Ok(UniqueJsonKeys)
            }

            fn visit_f64<E: de::Error>(self, _value: f64) -> Result<Self::Value, E> {
                Ok(UniqueJsonKeys)
            }

            fn visit_str<E: de::Error>(self, _value: &str) -> Result<Self::Value, E> {
                Ok(UniqueJsonKeys)
            }

            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueJsonKeys)
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                while seq.next_element::<UniqueJsonKeys>()?.is_some() {}
                Ok(UniqueJsonKeys)
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut keys = BTreeSet::new();
                while let Some(key) = map.next_key::<String>()? {
                    if !keys.insert(key) {
                        return Err(de::Error::custom("duplicate JSON key in reasoning source"));
                    }
                    map.next_value::<UniqueJsonKeys>()?;
                }
                Ok(UniqueJsonKeys)
            }
        }

        deserializer.deserialize_any(UniqueVisitor)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
// RG-02 wiring consumes this item; see verify_claude_encrypted_field.
#[allow(dead_code)]
struct ClaudeAssistantRecord<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
    message: ClaudeAssistantMessage<'a>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
// RG-02 wiring consumes this item; see verify_claude_encrypted_field.
#[allow(dead_code)]
struct ClaudeAssistantMessage<'a> {
    role: &'a str,
    #[serde(borrow)]
    content: Vec<&'a RawValue>,
}

#[derive(Deserialize)]
// RG-02 wiring consumes this item; see verify_claude_encrypted_field.
#[allow(dead_code)]
struct ClaudeBlockTag<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
// RG-02 wiring consumes this item; see verify_claude_encrypted_field.
#[allow(dead_code)]
struct ClaudeThinkingBlock<'a> {
    #[serde(rename = "type")]
    _kind: &'a str,
    #[serde(rename = "thinking", borrow)]
    thinking: &'a RawValue,
    #[serde(borrow)]
    signature: &'a RawValue,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
// RG-02 wiring consumes this item; see verify_claude_encrypted_field.
#[allow(dead_code)]
struct ClaudeRedactedThinkingBlock<'a> {
    #[serde(rename = "type")]
    _kind: &'a str,
    #[serde(borrow)]
    data: &'a RawValue,
}

#[allow(dead_code)] // RG-02 wiring consumes this helper; see verify_claude_encrypted_field.
fn json_string_field_bytes(raw: &RawValue) -> Option<&[u8]> {
    let bytes = raw.get().as_bytes();
    if bytes.len() < 2 || bytes.first() != Some(&b'"') || bytes.last() != Some(&b'"') {
        return None;
    }
    Some(&bytes[1..bytes.len() - 1])
}

/// Verify a single encrypted field in an authorized Claude assistant JSONL record.
///
/// The caller must first authorize and pin the source through the existing
/// provider-root/import path. This function verifies the official content
/// block shape; it does not authenticate a caller-supplied JSON document.
/// This type-contract verifier accepts the minimal synthetic envelope only;
/// real session metadata envelopes belong to the provider adapter contract.
/// Unknown or wrong-typed reasoning blocks fail closed without logging data.
/// No OpenCode field is registered: the pinned 2.0.24 `state` is an open record, not
/// proof of ciphertext. A selected non-reasoning block is also rejected;
/// callers classify ordinary text/tool blocks before invoking this verifier.
///
/// Visibility is `pub(crate)` (Codex re-review P1): the un-forgeable byte-type
/// guarantee requires that opaque construction is reachable only through the
/// authorized pipeline, not from arbitrary external callers synthesizing a
/// matching record.
#[allow(dead_code)] // RG-02 wiring consumes the verifier; see the structs above.
pub(crate) fn verify_claude_encrypted_field(
    source: &[u8],
    content_index: usize,
) -> Result<VerifiedOpaqueReasoning, ReasoningFieldError> {
    if u64::try_from(source.len()).map_or(true, |len| len > TRANSCRIPT_READ_HARD_CAP_BYTES) {
        return Err(ReasoningFieldError::SourceTooLarge);
    }
    serde_json::from_slice::<UniqueJsonKeys>(source)
        .map_err(|_| ReasoningFieldError::MalformedSource)?;
    let envelope: ClaudeAssistantRecord<'_> =
        serde_json::from_slice(source).map_err(|_| ReasoningFieldError::MalformedSource)?;
    if envelope.kind != "assistant" || envelope.message.role != "assistant" {
        return Err(ReasoningFieldError::UnsupportedShape);
    }
    let block = envelope
        .message
        .content
        .get(content_index)
        .ok_or(ReasoningFieldError::UnsupportedShape)?;
    let tag: ClaudeBlockTag<'_> =
        serde_json::from_str(block.get()).map_err(|_| ReasoningFieldError::UnsupportedShape)?;
    let (kind, raw) = match tag.kind {
        "thinking" => {
            let thinking: ClaudeThinkingBlock<'_> = serde_json::from_str(block.get())
                .map_err(|_| ReasoningFieldError::UnsupportedShape)?;
            if json_string_field_bytes(thinking.thinking).is_none() {
                return Err(ReasoningFieldError::UnsupportedShape);
            }
            (ReasoningSourceKind::Signature, thinking.signature)
        }
        "redacted_thinking" => {
            let redacted: ClaudeRedactedThinkingBlock<'_> = serde_json::from_str(block.get())
                .map_err(|_| ReasoningFieldError::UnsupportedShape)?;
            (ReasoningSourceKind::RedactedThinkingData, redacted.data)
        }
        _ => return Err(ReasoningFieldError::UnsupportedShape),
    };
    let data = json_string_field_bytes(raw).ok_or(ReasoningFieldError::UnsupportedShape)?;
    Ok(VerifiedOpaqueReasoning {
        record: ReasoningRecord {
            provider: ReasoningProvider::ClaudeCode,
            source_kind: Some(kind),
            availability: ReasoningAvailability::EncryptedUnavailable,
            warning: None,
        },
        bytes: OpaqueEncryptedBytes {
            data: data.to_vec(),
            provider: ReasoningProvider::ClaudeCode,
            source_kind: kind,
        },
    })
}

/// Verified provider ciphertext, never a redacted transcript or JSON value.
///
/// RG-01 exposes no conversion from arbitrary bytes. Construction happens
/// only inside this module after [`verify_claude_encrypted_field`] checks a
/// provider-declared source field. RG-02 will add its artifact sink separately.
///
/// ```compile_fail
/// use libra::internal::ai::observed_agents::reasoning::OpaqueEncryptedBytes;
/// let _: OpaqueEncryptedBytes = Vec::<u8>::new().into();
/// ```
///
/// ```compile_fail
/// use libra::internal::ai::observed_agents::reasoning::OpaqueEncryptedBytes;
/// let _: OpaqueEncryptedBytes = (&b"ciphertext"[..]).into();
/// ```
///
/// ```compile_fail
/// use libra::internal::ai::observed_agents::reasoning::OpaqueEncryptedBytes;
/// let payload: OpaqueEncryptedBytes = unreachable!();
/// let _ = serde_json::to_vec(&payload);
/// ```
///
/// ```compile_fail
/// use libra::internal::ai::observed_agents::reasoning::OpaqueEncryptedBytes;
/// let payload: OpaqueEncryptedBytes = unreachable!();
/// let _ = format!("{payload}");
/// ```
///
/// ```compile_fail
/// use libra::internal::ai::observed_agents::{RedactedBytes, reasoning::OpaqueEncryptedBytes};
/// let payload: OpaqueEncryptedBytes = unreachable!();
/// let _: RedactedBytes = payload.into();
/// ```
///
/// ```compile_fail
/// use libra::internal::ai::observed_agents::{coverage_digest_hex, reasoning::OpaqueEncryptedBytes};
/// let payload: OpaqueEncryptedBytes = unreachable!();
/// let _ = coverage_digest_hex(&[payload]);
/// ```
///
/// ```compile_fail
/// use libra::internal::ai::observed_agents::{Redactor, reasoning::OpaqueEncryptedBytes};
/// let payload: OpaqueEncryptedBytes = unreachable!();
/// let _ = Redactor::new_default().redact(&payload);
/// ```
#[derive(Clone)]
pub struct OpaqueEncryptedBytes {
    data: Vec<u8>,
    provider: ReasoningProvider,
    source_kind: ReasoningSourceKind,
}

impl OpaqueEncryptedBytes {
    /// Used only for isolated contract tests until verified provider wiring.
    /// `pub(crate)` so sibling lib test modules (history.rs RG-02 artifact
    /// tests) can construct verified-looking ciphertext for the writer.
    #[cfg(test)]
    pub(crate) fn from_verified_test_field(data: Vec<u8>) -> Self {
        Self {
            data,
            provider: ReasoningProvider::ClaudeCode,
            source_kind: ReasoningSourceKind::Signature,
        }
    }

    pub(crate) fn verified_source(&self) -> (ReasoningProvider, ReasoningSourceKind) {
        (self.provider, self.source_kind)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Hash the original wire bytes without decoding or serializing them.
    pub fn sha256_hex(&self) -> String {
        hex::encode(Sha256::digest(&self.data))
    }

    /// Byte-exact artifact view for the RG-02 checkpoint writer. `pub(crate)`
    /// so only the in-crate artifact sink (never an external caller) can
    /// obtain the raw ciphertext bytes for a single object write.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

impl fmt::Debug for OpaqueEncryptedBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpaqueEncryptedBytes")
            .field("byte_len", &self.data.len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn native_reasoning_proof_rejects_each_bound_dimension() {
        fn batch() -> NativeProvenReasoning {
            let prepared = synthetic_native_preparation(
                b"ordinary".to_vec(),
                carrier_test_support::CAPTURE,
                carrier_test_support::SOURCE,
                vec![b"opaque-wire".to_vec()],
            );
            let ReasoningSourceDisposition::NativeProven(batch) = prepared.disposition else {
                panic!("native fixture")
            };
            batch
        }
        let good = batch();
        good.validate(carrier_test_support::CAPTURE, ReasoningProvider::ClaudeCode)
            .expect("synthetic control");
        for dimension in 0..9 {
            let mut bad = batch();
            match dimension {
                0 => bad.proofs[0].issuer = NativeReasoningIssuer::SyntheticUnknown,
                1 => bad.proofs[0].provider = ReasoningProvider::Codex,
                2 => bad.proofs[0].source_session.push_str("wrong"),
                3 => bad.proofs[0].message += 1,
                4 => bad.proofs[0].part += 1,
                5 => bad.proofs[0].field = ReasoningSourceKind::RedactedThinkingData,
                6 => bad.proofs[0].original_wire_sha256[0] ^= 1,
                7 => bad.proofs[0].logical_turn.push_str("wrong"),
                8 => bad.capture_session.push_str("wrong"),
                _ => unreachable!(),
            }
            assert!(
                bad.validate(carrier_test_support::CAPTURE, ReasoningProvider::ClaudeCode)
                    .is_err(),
                "dimension {dimension}"
            );
        }
        assert!(
            good.validate("different-capture", ReasoningProvider::ClaudeCode)
                .is_err()
        );
        assert!(
            good.validate(carrier_test_support::CAPTURE, ReasoningProvider::OpenCode)
                .is_err()
        );
    }

    use std::time::Instant;

    use super::*;
    #[test]
    fn decryptor_registry_is_none_for_all_providers() {
        for provider in [
            ReasoningProvider::Codex,
            ReasoningProvider::ClaudeCode,
            ReasoningProvider::OpenCode,
        ] {
            assert!(reasoning_decryptor(provider).is_none());
            let state = if reasoning_decryptor(provider).is_none() {
                ReasoningAvailability::EncryptedUnavailable
            } else {
                panic!("no provider decryptor is registered");
            };
            assert_eq!(state, ReasoningAvailability::EncryptedUnavailable);
        }
    }

    #[test]
    fn contract_serde_snapshot() {
        let statuses = [
            ReasoningAvailability::ProviderVisible,
            ReasoningAvailability::EncryptedUnavailable,
            ReasoningAvailability::OpaqueArchived,
            ReasoningAvailability::NotPresent,
            ReasoningAvailability::UnsupportedShape,
        ];
        let absent = classify_unrecognized_field(
            ReasoningProvider::OpenCode,
            UnrecognizedReasoningField::Other,
            b"unrecognized",
        );
        let unknown = classify_unrecognized_field(
            ReasoningProvider::OpenCode,
            UnrecognizedReasoningField::ReasoningLike,
            b"unsupported",
        );
        let encrypted = ReasoningRecord {
            provider: ReasoningProvider::ClaudeCode,
            source_kind: Some(ReasoningSourceKind::Signature),
            availability: ReasoningAvailability::EncryptedUnavailable,
            warning: None,
        };
        // RG-04 additive freeze: the canonical bytes of a synthetic
        // SemanticRecord::Reasoning record are pinned alongside the metadata
        // contract. The variant's serde serialization is deliberately
        // fail-closed (ProviderVisibleText::serialize always errors), so the
        // canonical writer is the frozen persistence face.
        let (_, reasoning_text) = classify_provider_visible_reasoning(
            ReasoningProvider::OpenCode,
            ReasoningSourceKind::Reasoning,
            "synthetic".to_string(),
        );
        let reasoning_record =
            crate::internal::ai::observed_agents::coverage::SemanticRecord::Reasoning {
                provider: ReasoningProvider::OpenCode,
                source_kind: Some(ReasoningSourceKind::Reasoning),
                text: reasoning_text,
            };
        let reasoning_canonical = String::from_utf8(
            crate::internal::ai::observed_agents::coverage::canonical_turn_bytes(
                std::slice::from_ref(&reasoning_record),
            ),
        )
        .expect("canonical bytes are UTF-8");
        let projected = crate::internal::ai::observed_agents::coverage::safe_turn_projection(
            "opencode",
            &crate::internal::ai::observed_agents::coverage::NormalizedTurn {
                logical_turn_key: "fixture-turn".to_string(),
                ordinal: 0,
                completeness:
                    crate::internal::ai::observed_agents::coverage::Completeness::Complete,
                started_at: None,
                ended_at: None,
                records: vec![reasoning_record],
            },
        );
        let actual = serde_json::to_string_pretty(&serde_json::json!({
            "availability": statuses,
            "records": [absent, unknown, encrypted],
            "semantic_reasoning_canonical": reasoning_canonical,
            "semantic_reasoning_projection": projected,
            "semantic_reasoning_note": "RG-04 additive freeze: canonical bytes of a synthetic Reasoning record (write_canonical) and the full safe_turn_projection output; provider_visible text is redacted before any persistence and serde serialization of the variant is fail-closed.",
        }))
        .expect("fixed contract types serialize");
        assert_eq!(
            actual.trim(),
            include_str!("../../../../tests/fixtures/agent_transcripts/reasoning/contract.snap")
                .trim()
        );
    }

    #[test]
    fn undeclared_high_entropy_is_not_present() {
        for candidate in [
            "dGhpcy1sb29rcy1saWtlLWNpcGhlcnRleHQ=",
            "83fde0f5e4c1208fddc5591f95befd2d9f45ca37",
        ] {
            let classified = classify_unrecognized_field(
                ReasoningProvider::OpenCode,
                UnrecognizedReasoningField::Other,
                candidate.as_bytes(),
            );
            assert_eq!(classified.availability(), ReasoningAvailability::NotPresent);
            assert_eq!(classified.warning(), None);
        }
    }

    #[test]
    fn debug_output_has_no_payload() {
        let payload = OpaqueEncryptedBytes::from_verified_test_field(
            b"CANARY_PRIVATE_CIPHERTEXT_4d5c".to_vec(),
        );
        let debug = format!("{payload:?}");
        assert!(!debug.contains("CANARY_PRIVATE_CIPHERTEXT_4d5c"));
        assert!(!debug.contains("43414e415259"));
        assert_eq!(payload.len(), 30);
        assert!(!payload.is_empty());
    }

    #[test]
    fn unsupported_shape_yields_warning() {
        let classified = classify_unrecognized_field(
            ReasoningProvider::OpenCode,
            UnrecognizedReasoningField::ReasoningLike,
            b"CANARY_PRIVATE_REASONING",
        );
        assert_eq!(
            classified.availability(),
            ReasoningAvailability::UnsupportedShape
        );
        assert_eq!(
            classified.warning(),
            Some(ReasoningWarning::UnsupportedShape)
        );
        let diagnostic = serde_json::to_string(&classified).expect("fixed diagnostic");
        assert!(!diagnostic.contains("payload"));
        assert!(!diagnostic.contains("field_path"));
    }

    #[test]
    fn verified_claude_json_string_bytes_exact() {
        let source = br#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"thinking","thinking":"","signature":"A\u0042\\C"},{"type":"redacted_thinking","data":"\u0041/raw=="}]}}"#;
        let signature = verify_claude_encrypted_field(source, 0)
            .expect("declared encrypted thinking signature");
        assert_eq!(
            signature.record().availability(),
            ReasoningAvailability::EncryptedUnavailable
        );
        assert_eq!(
            signature.record().source_kind(),
            Some(ReasoningSourceKind::Signature)
        );
        assert_eq!(signature.into_opaque().data, br"A\u0042\\C");

        let redacted = verify_claude_encrypted_field(source, 1)
            .expect("declared encrypted redacted thinking data");
        assert_eq!(
            redacted.record().source_kind(),
            Some(ReasoningSourceKind::RedactedThinkingData)
        );
        assert_eq!(
            serde_json::to_value(redacted.record())
                .expect("safe metadata")
                .get("source_kind"),
            Some(&serde_json::json!("redacted_thinking_data"))
        );
        assert_eq!(redacted.into_opaque().data, br"\u0041/raw==");
    }

    #[test]
    fn unverified_field_rejected_without_payload() {
        for source in [
            &br#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"thinking","thinking":"x","signature":123}]}}"#[..],
            &br#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"thinking","thinking":"x","signature":"CANARY_PRIVATE_REASONING","signature":"again"}]}}"#[..],
            &br#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"thinking","thinking":"x","signature":"CANARY_PRIVATE_REASONING","extra":"value"}]}}"#[..],
            &br#"{"type":"assistant","debug":{"x":1,"x":2},"message":{"role":"assistant","content":[{"type":"thinking","thinking":"x","signature":"CANARY_PRIVATE_REASONING"}]}}"#[..],
            &br#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"thinking","thinking":"x","signature":"CANARY_PRIVATE_REASONING"}]} ,"debug":{"a":1,"\u0061":2}}"#[..],
            &br#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"future_thinking","encrypted":"CANARY_PRIVATE_REASONING"}]}}"#[..],
            &br#"{"type":"assistant","content":[{"type":"reasoning","text":"x","state":{"anthropic":{"signature":"CANARY_PRIVATE_REASONING"}}}]}"#[..],
            &br#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"CANARY_PRIVATE_REASONING","signature":"fake"}]}}"#[..],
            &br#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"ordinary text"}]}}"#[..],
            &br#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"tool_use","id":"call_1","name":"Read","input":{}}]}}"#[..],
        ] {
            let error = verify_claude_encrypted_field(source, 0).expect_err("unverified field");
            assert!(!format!("{error:?}").contains("CANARY_PRIVATE_REASONING"));
        }
        let oversized = vec![b'x'; TRANSCRIPT_READ_HARD_CAP_BYTES as usize + 1];
        assert_eq!(
            verify_claude_encrypted_field(&oversized, 0).expect_err("bounded source"),
            ReasoningFieldError::SourceTooLarge
        );
    }

    #[test]
    fn readable_text_cannot_bypass_classification_or_serde() {
        let canary = "sk-ant-EXAMPLE0000000000000000";
        let (_, text) = classify_provider_visible_reasoning(
            ReasoningProvider::OpenCode,
            ReasoningSourceKind::Reasoning,
            canary.to_string(),
        );
        assert_eq!(text.as_str(), canary);
        let record = crate::internal::ai::observed_agents::coverage::SemanticRecord::Reasoning {
            provider: ReasoningProvider::OpenCode,
            source_kind: Some(ReasoningSourceKind::Reasoning),
            text,
        };
        let error = serde_json::to_value(&record).expect_err("unredacted serde must fail closed");
        assert!(!error.to_string().contains(canary));
        assert!(serde_json::from_str::<ProviderVisibleText>("\"synthetic\"").is_err());
        for availability in [
            ReasoningAvailability::EncryptedUnavailable,
            ReasoningAvailability::OpaqueArchived,
            ReasoningAvailability::NotPresent,
            ReasoningAvailability::UnsupportedShape,
        ] {
            let classification = ReasoningRecord {
                provider: ReasoningProvider::OpenCode,
                source_kind: None,
                availability,
                warning: None,
            };
            assert!(
                ProviderVisibleText::from_classification(&classification, canary.to_string())
                    .is_none()
            );
        }
    }

    #[test]
    fn duplicate_keys_in_unselected_block_are_rejected() {
        let source = br#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"first","text":"second"},{"type":"thinking","thinking":"","signature":"CANARY_PRIVATE_REASONING"}]}}"#;
        let error = verify_claude_encrypted_field(source, 1)
            .expect_err("duplicate key in an unselected sibling must fail closed");
        assert_eq!(error, ReasoningFieldError::MalformedSource);
        assert!(!format!("{error:?}").contains("CANARY_PRIVATE_REASONING"));
        let unique = br#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"first"},{"type":"thinking","thinking":"","signature":"CANARY_PRIVATE_REASONING"}]}}"#;
        assert!(verify_claude_encrypted_field(unique, 1).is_ok());
    }

    #[test]
    fn reasoning_field_error_display_is_stable() {
        assert_eq!(TRANSCRIPT_READ_HARD_CAP_BYTES, 16 * 1024 * 1024);
        for (error, expected) in [
            (
                ReasoningFieldError::SourceTooLarge,
                "reasoning source exceeds the 16 MiB transcript read limit; use a bounded export",
            ),
            (
                ReasoningFieldError::MalformedSource,
                "reasoning source is not a valid provider record; check the pinned provider export",
            ),
            (
                ReasoningFieldError::UnsupportedShape,
                "reasoning field has an unsupported shape; update the pinned provider contract",
            ),
        ] {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    #[ignore = "run explicitly in release mode on the acceptance machine"]
    fn verified_source_16_mib_budget() {
        let mut source = b"{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"redacted_thinking\",\"data\":\"".to_vec();
        let suffix = b"\"}]}}";
        let fill =
            (TRANSCRIPT_READ_HARD_CAP_BYTES as usize).saturating_sub(source.len() + suffix.len());
        assert!(fill > 0);
        source.extend(std::iter::repeat_n(b'A', fill));
        source.extend_from_slice(suffix);
        assert_eq!(source.len(), TRANSCRIPT_READ_HARD_CAP_BYTES as usize);
        let started = Instant::now();
        let ciphertext = verify_claude_encrypted_field(&source, 0)
            .expect("synthetic pinned encrypted field")
            .into_opaque();
        std::hint::black_box(ciphertext.sha256_hex());
        let elapsed = started.elapsed();
        eprintln!("RG-01 verified-source + SHA-256 16 MiB: {elapsed:?}");
        #[cfg(not(debug_assertions))]
        assert!(
            elapsed <= std::time::Duration::from_millis(50),
            "RG-01 16 MiB source/hash budget exceeded: {elapsed:?}"
        );
    }

    #[test]
    fn opaque_digest_uses_original_bytes() {
        let payload = OpaqueEncryptedBytes::from_verified_test_field(b"abc".to_vec());
        assert_eq!(
            payload.sha256_hex(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn opaque_digest_stays_out_of_metadata() {
        let payload = OpaqueEncryptedBytes::from_verified_test_field(b"private bytes".to_vec());
        let digest = payload.sha256_hex();
        let metadata = classify_unrecognized_field(
            ReasoningProvider::OpenCode,
            UnrecognizedReasoningField::Other,
            &payload.data,
        );
        let json = serde_json::to_string(&metadata).expect("safe metadata");
        assert!(!json.contains(&digest));
        assert!(!json.contains("private bytes"));
    }
}
