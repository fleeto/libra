//! Pure, bounded native-origin wire codec. Parsed values remain untrusted.

use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;
use url::Url;

use super::super::coverage::parse_canon_value;

pub const PRODUCER_VERSION: &str = "2.0.26-libra-native.1";
pub const RECEIPT_BYTE_CAP: usize = 8192;
pub const DESCRIPTOR_BYTE_CAP: usize = 65536;
const TRANSPORT_POLICY: &str = "compiled-native-http-only-v1";

/// A fixed, actionable diagnostic that never echoes untrusted wire contents.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("Invalid native-origin wire: {reason}; discard the proof and keep the content partial")]
pub struct CodecError {
    reason: &'static str,
}

fn invalid(reason: &'static str) -> CodecError {
    CodecError { reason }
}

/// Wire protocol identity, not an approved transport or endpoint.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Protocol {
    #[serde(rename = "anthropic-messages")]
    AnthropicMessages,
    #[serde(rename = "bedrock-converse")]
    BedrockConverse,
    #[serde(rename = "open-responses")]
    OpenResponses,
    #[serde(rename = "openai-responses")]
    OpenaiResponses,
}

impl Protocol {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AnthropicMessages => "anthropic-messages",
            Self::BedrockConverse => "bedrock-converse",
            Self::OpenResponses => "open-responses",
            Self::OpenaiResponses => "openai-responses",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum FieldName {
    #[serde(rename = "signature")]
    Signature,
    #[serde(rename = "redactedData")]
    RedactedData,
    #[serde(rename = "reasoningEncryptedContent")]
    ReasoningEncryptedContent,
}

impl FieldName {
    fn tag(self) -> u32 {
        match self {
            Self::Signature => 1,
            Self::RedactedData => 2,
            Self::ReasoningEncryptedContent => 3,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FieldDigest {
    pub name: FieldName,
    pub length: u32,
    pub sha256: String,
}

/// Public fields are untrusted wire data. Serde decoding alone does not validate
/// the contract; use `decode_receipt` and always revalidate with `signing_bytes`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Receipt {
    pub version: u32,
    pub issuer: String,
    pub key_generation: u32,
    pub build_identity: String,
    pub producer_version: String,
    pub policy_digest: String,
    pub protocol: Protocol,
    pub route: String,
    pub endpoint_digest: String,
    pub source_session: String,
    pub source_message: String,
    pub source_part: String,
    pub stream: String,
    pub fragment: String,
    pub fields: Vec<FieldDigest>,
    pub signature: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    pub version: u32,
    pub protocol: Protocol,
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Policy {
    pub version: u32,
    pub build_identity: String,
    pub protocols: Vec<Protocol>,
    pub endpoint_digests: Vec<String>,
    pub builtin_set_digest: String,
    pub libra_template_digest: String,
    pub libra_rendered_digest: String,
    pub libra_command_path: String,
    pub event_map_digest: String,
    pub export_deadline_millis: u32,
    pub transport_policy: String,
}

/// Reject duplicates on the original bytes, before any ordinary JSON projection.
/// The strict intermediate value is discarded: decoding the original bytes again
/// preserves numeric grammar and avoids float-based normalization.
fn raw<T: DeserializeOwned>(input: &[u8], cap: usize) -> Result<T, CodecError> {
    if input.len() > cap {
        return Err(invalid("JSON byte cap"));
    }
    parse_canon_value(input).map_err(|_| invalid("JSON grammar or repeated member"))?;
    serde_json::from_slice(input).map_err(|_| invalid("closed grammar"))
}

pub fn decode_receipt(input: &[u8]) -> Result<Receipt, CodecError> {
    let value: Receipt = raw(input, RECEIPT_BYTE_CAP)?;
    validate_receipt(&value)?;
    Ok(value)
}

pub fn decode_endpoint(input: &[u8]) -> Result<Endpoint, CodecError> {
    let value: Endpoint = raw(input, DESCRIPTOR_BYTE_CAP)?;
    validate_endpoint(&value)?;
    Ok(value)
}

pub fn decode_policy(input: &[u8]) -> Result<Policy, CodecError> {
    let value: Policy = raw(input, DESCRIPTOR_BYTE_CAP)?;
    validate_policy(&value)?;
    Ok(value)
}

fn text(value: &str, cap: usize) -> Result<(), CodecError> {
    if value.is_empty() || value.len() > cap || value.chars().any(char::is_control) {
        return Err(invalid("UTF-8 text or byte length"));
    }
    Ok(())
}

fn hexadecimal(value: &str, bytes: usize) -> Result<(), CodecError> {
    if value.len() != bytes * 2
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(invalid("lowercase hex grammar"));
    }
    Ok(())
}

fn json_size<T: Serialize>(value: &T, cap: usize) -> Result<(), CodecError> {
    // Per-field caps are checked before serialization, bounding this allocation.
    let encoded = serde_json::to_vec(value).map_err(|_| invalid("JSON encoding"))?;
    if encoded.len() > cap {
        return Err(invalid("JSON byte cap"));
    }
    Ok(())
}

fn validate_receipt(value: &Receipt) -> Result<(), CodecError> {
    if value.version != 1 || value.key_generation == 0 || value.producer_version != PRODUCER_VERSION
    {
        return Err(invalid("receipt version or key generation"));
    }
    hexadecimal(&value.issuer, 32)?;
    hexadecimal(&value.build_identity, 16)?;
    hexadecimal(&value.policy_digest, 32)?;
    hexadecimal(&value.endpoint_digest, 32)?;
    hexadecimal(&value.stream, 16)?;
    let part = value
        .source_part
        .strip_prefix("npart_")
        .ok_or_else(|| invalid("source part"))?;
    hexadecimal(part, 16)?;
    text(&value.route, 256)?;
    text(&value.source_session, 1024)?;
    text(&value.source_message, 1024)?;
    text(&value.fragment, 1024)?;
    if value.fields.is_empty() || value.fields.len() > 3 {
        return Err(invalid("array bounds"));
    }
    if value
        .fields
        .windows(2)
        .any(|pair| pair[0].name >= pair[1].name)
    {
        return Err(invalid("field order or repetition"));
    }
    for field in &value.fields {
        if field.length == 0 {
            return Err(invalid("field length"));
        }
        hexadecimal(&field.sha256, 32)?;
    }
    if value.signature.len() != 88 {
        return Err(invalid("signature encoding"));
    }
    let signature = STANDARD
        .decode(&value.signature)
        .map_err(|_| invalid("signature encoding"))?;
    if signature.len() != 64 || STANDARD.encode(&signature) != value.signature {
        return Err(invalid("signature encoding"));
    }
    json_size(value, RECEIPT_BYTE_CAP)
}

fn validate_endpoint(value: &Endpoint) -> Result<(), CodecError> {
    if value.version != 1 {
        return Err(invalid("endpoint version"));
    }
    text(&value.url, DESCRIPTOR_BYTE_CAP)?;
    if !value.url.bytes().all(|b| (0x21..=0x7e).contains(&b)) || value.url.contains(['?', '#']) {
        return Err(invalid("endpoint URL spelling"));
    }
    let url = Url::parse(&value.url).map_err(|_| invalid("endpoint URL"))?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.as_str() != value.url
    {
        return Err(invalid("endpoint URL spelling"));
    }
    json_size(value, DESCRIPTOR_BYTE_CAP)
}

fn validate_policy(value: &Policy) -> Result<(), CodecError> {
    if value.version != 1
        || value.export_deadline_millis != 30000
        || value.transport_policy != TRANSPORT_POLICY
    {
        return Err(invalid("policy version or transport"));
    }
    hexadecimal(&value.build_identity, 16)?;
    if value.protocols.is_empty()
        || value.protocols.len() > 4
        || value.endpoint_digests.is_empty()
        || value.endpoint_digests.len() > 32
    {
        return Err(invalid("array bounds"));
    }
    if value.protocols.windows(2).any(|pair| pair[0] >= pair[1])
        || value
            .endpoint_digests
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        return Err(invalid("list order or repetition"));
    }
    for digest in &value.endpoint_digests {
        hexadecimal(digest, 32)?;
    }
    for digest in [
        &value.builtin_set_digest,
        &value.libra_template_digest,
        &value.libra_rendered_digest,
        &value.event_map_digest,
    ] {
        hexadecimal(digest, 32)?;
    }
    text(&value.libra_command_path, 4096)?;
    if !value.libra_command_path.starts_with('/') {
        return Err(invalid("absolute Libra command path"));
    }
    json_size(value, DESCRIPTOR_BYTE_CAP)
}

struct Encoder {
    bytes: Vec<u8>,
    cap: usize,
}

impl Encoder {
    fn new(domain: &[u8], cap: usize) -> Result<Self, CodecError> {
        let mut value = Self {
            bytes: Vec::new(),
            cap,
        };
        value.append(domain)?;
        Ok(value)
    }

    fn append(&mut self, value: &[u8]) -> Result<(), CodecError> {
        if self
            .bytes
            .len()
            .checked_add(value.len())
            .is_none_or(|length| length > self.cap)
        {
            return Err(invalid("canonical byte cap"));
        }
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    fn u32(&mut self, value: u32) -> Result<(), CodecError> {
        self.append(&value.to_be_bytes())
    }

    fn lp(&mut self, value: &str) -> Result<(), CodecError> {
        self.u32(u32::try_from(value.len()).map_err(|_| invalid("UTF-8 byte length"))?)?;
        self.append(value.as_bytes())
    }

    fn hex(&mut self, value: &str) -> Result<(), CodecError> {
        for pair in value.as_bytes().as_chunks::<2>().0 {
            let high = char::from(pair[0])
                .to_digit(16)
                .ok_or_else(|| invalid("hex encoding"))?;
            let low = char::from(pair[1])
                .to_digit(16)
                .ok_or_else(|| invalid("hex encoding"))?;
            self.append(&[(high * 16 + low) as u8])?;
        }
        Ok(())
    }
}

/// Canonical signature preimage only. This never signs, verifies, trusts or
/// constructs NativeProven/OpaqueEncryptedBytes, even with a valid wire signature.
pub fn signing_bytes(value: &Receipt) -> Result<Vec<u8>, CodecError> {
    validate_receipt(value)?;
    let mut out = Encoder::new(
        b"libra-opencode-native-origin-receipt-v1\0",
        RECEIPT_BYTE_CAP,
    )?;
    out.u32(value.version)?;
    out.hex(&value.issuer)?;
    out.u32(value.key_generation)?;
    out.hex(&value.build_identity)?;
    out.lp(&value.producer_version)?;
    out.hex(&value.policy_digest)?;
    out.lp(value.protocol.as_str())?;
    out.lp(&value.route)?;
    out.hex(&value.endpoint_digest)?;
    out.lp(&value.source_session)?;
    out.lp(&value.source_message)?;
    out.lp(&value.source_part)?;
    out.hex(&value.stream)?;
    out.lp(&value.fragment)?;
    out.u32(u32::try_from(value.fields.len()).map_err(|_| invalid("array bounds"))?)?;
    for field in &value.fields {
        out.u32(field.name.tag())?;
        out.u32(field.length)?;
        out.hex(&field.sha256)?;
    }
    Ok(out.bytes)
}

pub fn endpoint_bytes(value: &Endpoint) -> Result<Vec<u8>, CodecError> {
    validate_endpoint(value)?;
    let mut out = Encoder::new(b"libra-opencode-native-endpoint-v1\0", DESCRIPTOR_BYTE_CAP)?;
    out.u32(value.version)?;
    out.lp(value.protocol.as_str())?;
    out.lp(&value.url)?;
    Ok(out.bytes)
}

pub fn policy_bytes(value: &Policy) -> Result<Vec<u8>, CodecError> {
    validate_policy(value)?;
    let mut out = Encoder::new(b"libra-opencode-native-policy-v1\0", DESCRIPTOR_BYTE_CAP)?;
    out.u32(value.version)?;
    out.hex(&value.build_identity)?;
    out.u32(u32::try_from(value.protocols.len()).map_err(|_| invalid("array bounds"))?)?;
    for protocol in &value.protocols {
        out.lp(protocol.as_str())?;
    }
    out.u32(u32::try_from(value.endpoint_digests.len()).map_err(|_| invalid("array bounds"))?)?;
    for digest in &value.endpoint_digests {
        out.hex(digest)?;
    }
    out.hex(&value.builtin_set_digest)?;
    out.hex(&value.libra_template_digest)?;
    out.hex(&value.libra_rendered_digest)?;
    out.lp(&value.libra_command_path)?;
    out.hex(&value.event_map_digest)?;
    out.u32(value.export_deadline_millis)?;
    out.lp(&value.transport_policy)?;
    Ok(out.bytes)
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};

    use super::*;

    fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../../../../../tests/fixtures/agent_transcripts/native_origin_receipt_v1.json"
        ))
        .expect("synthetic fixture")
    }

    fn wire(value: &Value) -> Vec<u8> {
        serde_json::to_vec(value).expect("fixture JSON")
    }
    fn receipt() -> Value {
        fixture()["vectors"][0]["receipt"].clone()
    }
    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
    fn rejected(value: &Value) {
        let error = decode_receipt(&wire(value)).expect_err("invalid receipt must fail");
        assert!(error.to_string().starts_with("Invalid native-origin wire:"));
    }

    #[test]
    fn npr_02_canonical() {
        let data = fixture();
        for vector in data["vectors"].as_array().expect("vectors") {
            let parsed = decode_receipt(&wire(&vector["receipt"])).expect("wire syntax only");
            let bytes = signing_bytes(&parsed).expect("canonical");
            assert_eq!(hex(&bytes), vector["canonical_hex"].as_str().unwrap());
            assert_eq!(
                hex(&Sha256::digest(&bytes)),
                vector["canonical_sha256"].as_str().unwrap()
            );
            assert_eq!(
                bytes.len(),
                vector["canonical_bytes"].as_u64().unwrap() as usize
            );
            assert!(bytes.starts_with(b"libra-opencode-native-origin-receipt-v1\0\0\0\0\x01"));
            let mut changed = parsed.clone();
            changed.signature = STANDARD.encode([7u8; 64]);
            assert_eq!(
                signing_bytes(&changed).unwrap(),
                bytes,
                "signature excluded"
            );
            changed.fragment.push('x');
            assert_ne!(
                signing_bytes(&changed).unwrap(),
                bytes,
                "coordinates included"
            );
            let reversed = vector["receipt"]
                .as_object()
                .unwrap()
                .iter()
                .rev()
                .map(|(key, value)| format!("{}:{}", serde_json::to_string(key).unwrap(), value))
                .collect::<Vec<_>>()
                .join(",");
            assert_eq!(
                signing_bytes(&decode_receipt(format!("{{{reversed}}}").as_bytes()).unwrap())
                    .unwrap(),
                bytes
            );
        }
        for vector in data["policies"].as_array().unwrap() {
            let bytes = policy_bytes(&decode_policy(&wire(&vector["policy"])).unwrap()).unwrap();
            assert_eq!(hex(&bytes), vector["canonical_hex"].as_str().unwrap());
            assert_eq!(
                hex(&Sha256::digest(&bytes)),
                vector["canonical_sha256"].as_str().unwrap()
            );
        }
        for vector in data["endpoints"].as_array().unwrap() {
            let bytes =
                endpoint_bytes(&decode_endpoint(&wire(&vector["endpoint"])).unwrap()).unwrap();
            assert_eq!(hex(&bytes), vector["canonical_hex"].as_str().unwrap());
            assert_eq!(
                hex(&Sha256::digest(&bytes)),
                vector["canonical_sha256"].as_str().unwrap()
            );
        }
    }

    #[test]
    fn npr_02_grammar() {
        for (key, value) in [
            ("version", json!(2)),
            ("keyGeneration", json!(0)),
            ("keyGeneration", json!(4294967296u64)),
            ("keyGeneration", json!(-1)),
            ("keyGeneration", json!(1.5)),
            ("issuer", json!("A".repeat(64))),
            ("issuer", json!(format!("{}\n", "a".repeat(64)))),
            ("stream", json!(format!("{}\n", "a".repeat(32)))),
            ("sourcePart", json!(format!("npart_{}\n", "a".repeat(32)))),
            ("producerVersion", json!("2.0.24-libra-native.1")),
            ("protocol", json!("arbitrary-provider")),
            ("fragment", Value::Null),
            (
                "signature",
                json!(format!("{}\n", STANDARD.encode([0u8; 64]))),
            ),
            ("signature", json!(format!("{}B==", "A".repeat(85)))),
            ("extra", json!(true)),
        ] {
            let mut value_wire = receipt();
            value_wire[key] = value;
            rejected(&value_wire);
        }
        let mut v = receipt();
        v["fields"][0]["length"] = json!(0);
        rejected(&v);
        v = receipt();
        v["fields"][0]["extra"] = json!(true);
        rejected(&v);
        v = receipt();
        v["fields"][0]["name"] = json!("unknown");
        rejected(&v);
        v = fixture()["vectors"][2]["receipt"].clone();
        v["fields"].as_array_mut().unwrap().swap(0, 1);
        rejected(&v);
        v = receipt();
        v["fields"] = json!([v["fields"][0].clone(), v["fields"][0].clone()]);
        rejected(&v);
        for url in [
            "http://synthetic.invalid/a",
            "https://user@synthetic.invalid/a",
            "https://synthetic.invalid/a?",
            "https://synthetic.invalid/a#",
            "https://synthetic.invalid:443/a",
            "https://SYNTHETIC.invalid/a",
            "https://synthetic.invalid",
            "https://synthetic.invalid/a/../b",
            "https://synthetic.invalid/汉字",
            "https://synthetic.invalid/a\\b",
        ] {
            let endpoint = json!({"version":1,"protocol":"openai-responses","url":url});
            assert!(decode_endpoint(&wire(&endpoint)).is_err(), "URL {url}");
        }
        for (key, value) in [
            ("exportDeadlineMillis", json!(29999)),
            ("transportPolicy", json!("custom")),
            ("libraCommandPath", json!("relative/libra")),
            ("approved", json!(true)),
            (
                "protocols",
                json!(["openai-responses", "anthropic-messages"]),
            ),
            ("endpointDigests", json!(["a".repeat(64), "a".repeat(64)])),
        ] {
            let mut p = fixture()["policies"][0]["policy"].clone();
            p[key] = value;
            assert!(decode_policy(&wire(&p)).is_err(), "policy {key}");
        }
    }

    #[test]
    fn npr_02_raw_duplicates() {
        let compact = String::from_utf8(wire(&receipt())).unwrap();
        for raw in [
            compact.replacen("{", "{\"issuer\":\"do-not-echo-secret\",", 1),
            compact.replacen("\"version\":1", "\"version\":1,\"\\u0076ersion\":1", 1),
            compact.replacen("\"length\":39", "\"length\":39,\"length\":39", 1),
        ] {
            let error = decode_receipt(raw.as_bytes()).unwrap_err();
            assert_eq!(error.reason, "JSON grammar or repeated member");
            assert!(!error.to_string().contains("do-not-echo-secret"));
        }
        let p = String::from_utf8(wire(&fixture()["policies"][0]["policy"])).unwrap();
        assert!(decode_policy(p.replacen("{", "{\"version\":1,", 1).as_bytes()).is_err());
        let e = String::from_utf8(wire(&fixture()["endpoints"][0]["endpoint"])).unwrap();
        assert!(decode_endpoint(e.replacen("{", "{\"version\":1,", 1).as_bytes()).is_err());
        assert!(decode_receipt(format!("{compact} null").as_bytes()).is_err());
        assert!(
            decode_receipt(
                compact
                    .replacen("\"keyGeneration\":1", "\"keyGeneration\":1.0", 1)
                    .as_bytes()
            )
            .is_err()
        );
    }

    #[test]
    fn npr_02_utf8() {
        let mut value = receipt();
        value["fragment"] = json!("e\u{301}汉字🧪");
        let parsed = decode_receipt(&wire(&value)).unwrap();
        assert_eq!(parsed.fragment.as_bytes(), "e\u{301}汉字🧪".as_bytes());
        let canonical = signing_bytes(&parsed).unwrap();
        assert!(
            canonical
                .windows(parsed.fragment.len())
                .any(|w| w == parsed.fragment.as_bytes())
        );
        let mut normalized = parsed.clone();
        normalized.fragment = "é汉字🧪".into();
        assert_ne!(signing_bytes(&normalized).unwrap(), canonical);
        let compact = String::from_utf8(wire(&receipt())).unwrap();
        for escape in ["\\ud800", "\\udfff", "\\u0000", "\\u0085"] {
            let mut v = receipt();
            v["fragment"] = json!("placeholder");
            let raw = String::from_utf8(wire(&v))
                .unwrap()
                .replace("placeholder", escape);
            assert!(decode_receipt(raw.as_bytes()).is_err());
        }
        let mut invalid_bytes = compact.into_bytes();
        invalid_bytes[0] = 0xff;
        assert!(decode_receipt(&invalid_bytes).is_err());
        value["fragment"] = json!("汉".repeat(341));
        assert!(decode_receipt(&wire(&value)).is_ok());
        value["fragment"] = json!("汉".repeat(342));
        rejected(&value);
    }

    #[test]
    fn npr_02_bounds() {
        let mut raw = wire(&receipt());
        raw.resize(RECEIPT_BYTE_CAP, b' ');
        assert!(decode_receipt(&raw).is_ok());
        raw.push(b' ');
        assert_eq!(decode_receipt(&raw).unwrap_err().reason, "JSON byte cap");
        assert_eq!(
            decode_receipt(&vec![b' '; 1_000_000]).unwrap_err().reason,
            "JSON byte cap"
        );
        for fields in [json!([]), json!(vec![receipt()["fields"][0].clone(); 4])] {
            let mut v = receipt();
            v["fields"] = fields;
            rejected(&v);
        }
        let mut v = decode_receipt(&wire(&receipt())).unwrap();
        v.fields[0].length = u32::MAX;
        assert!(
            signing_bytes(&v).unwrap().len() < RECEIPT_BYTE_CAP,
            "declared payload length grants no allocation"
        );
        v.route = "x".repeat(257);
        assert!(signing_bytes(&v).is_err());
        let mut p = decode_policy(&wire(&fixture()["policies"][0]["policy"])).unwrap();
        p.endpoint_digests = vec!["a".repeat(64); 33];
        assert!(policy_bytes(&p).is_err());
        let mut endpoint = decode_endpoint(&wire(&fixture()["endpoints"][0]["endpoint"])).unwrap();
        endpoint.url = "x".repeat(DESCRIPTOR_BYTE_CAP + 1);
        assert!(endpoint_bytes(&endpoint).is_err());
        assert_eq!(
            decode_endpoint(&vec![b' '; DESCRIPTOR_BYTE_CAP + 1])
                .unwrap_err()
                .reason,
            "JSON byte cap"
        );
        assert_eq!(
            decode_policy(&vec![b' '; DESCRIPTOR_BYTE_CAP + 1])
                .unwrap_err()
                .reason,
            "JSON byte cap"
        );
        let mut encoder = Encoder::new(b"", 4).unwrap();
        encoder.u32(1).unwrap();
        assert_eq!(
            encoder.append(&[1]).unwrap_err().reason,
            "canonical byte cap"
        );
    }

    #[test]
    fn npr_02_no_authority() {
        let parsed = decode_receipt(&wire(&receipt())).unwrap();
        assert_eq!(STANDARD.decode(&parsed.signature).unwrap(), vec![0u8; 64]);
        let before = parsed.clone();
        let bytes = signing_bytes(&parsed).unwrap();
        assert_eq!(parsed, before);
        assert!(!bytes.is_empty());
        let error = invalid("closed grammar");
        assert_eq!(
            error.to_string(),
            "Invalid native-origin wire: closed grammar; discard the proof and keep the content partial"
        );
        // A syntactically valid all-zero signature is deliberately not checked
        // cryptographically here and never constructs a verified provenance type.
        let mut mutated = parsed.clone();
        mutated.issuer = "bad".into();
        assert!(
            signing_bytes(&mutated).is_err(),
            "public wire mutations are revalidated"
        );
    }
}
