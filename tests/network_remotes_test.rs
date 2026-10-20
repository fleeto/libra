#![cfg(feature = "test-network")]
#![recursion_limit = "256"]

use futures_util::StreamExt;
use libra::{
    git_protocol::ServiceType::UploadPack,
    internal::protocol::{
        ProtocolClient,
        https_client::HttpsClient,
        lfs_client::{LFSClient, LfsBatchResponse},
    },
    lfs_structs::{BatchRequest, Operation, RequestObject},
    utils::lfs,
};
use url::Url;

const PUBLIC_FIXTURE_REPO: &str = "https://github.com/libra-tools/libra.git";
const PUBLIC_FIXTURE_REPO_WEB_URL: &str = "https://github.com/libra-tools/libra/";

#[tokio::test]
async fn https_discovery_upload_pack_lists_refs() {
    let client = HttpsClient::from_url(&Url::parse(PUBLIC_FIXTURE_REPO).unwrap());
    let discovery = client
        .discovery_reference(UploadPack)
        .await
        .expect("discovery should succeed against public GitHub repo");

    assert!(!discovery.refs.is_empty(), "expected advertised refs");
    assert!(
        discovery
            .refs
            .iter()
            .any(|reference| reference.name().starts_with("refs/heads/")),
        "expected at least one branch ref"
    );
}

#[tokio::test]
async fn https_upload_pack_returns_pack_data() {
    let client = HttpsClient::from_url(&Url::parse(PUBLIC_FIXTURE_REPO_WEB_URL).unwrap());
    let discovery = client
        .discovery_reference(UploadPack)
        .await
        .expect("discovery should succeed against public GitHub repo");
    let main_ref = discovery
        .refs
        .iter()
        .find(|reference| reference.name() == "refs/heads/main")
        .expect("expected stable main branch ref");
    let want = vec![main_ref.hash().to_string()];

    let have = Vec::new();
    let mut result_stream = client
        .fetch_objects(&have, &want, &[], Some(1))
        .await
        .expect("upload-pack request should succeed");

    // The wire check only needs the pack signature, not a complete download
    // of a public repository whose pack size grows over time.
    const MAX_PACK_PREFIX: usize = 1024 * 1024;
    let mut buffer = Vec::new();
    while let Some(item) = result_stream.next().await {
        let chunk = item.expect("pack stream chunk should be readable");
        let remaining = MAX_PACK_PREFIX - buffer.len();
        buffer.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
        if buffer
            .windows(8)
            .any(|window| window == b"PACK\0\0\0\x02" || window == b"PACK\0\0\0\x03")
        {
            return;
        }
        assert!(
            buffer.len() < MAX_PACK_PREFIX,
            "upload-pack response did not contain a PACK v2/v3 header in its first {MAX_PACK_PREFIX} bytes"
        );
    }
    panic!("upload-pack response should contain a PACK v2/v3 header");
}

#[tokio::test]
async fn github_lfs_batch_download_returns_response() {
    let batch_request = BatchRequest {
        operation: Operation::Download,
        transfers: vec![lfs::LFS_TRANSFER_API.to_string()],
        objects: vec![RequestObject {
            oid: "01cb1483670f1c497412f25f9f8f7dde31a8fab0960291035af03939ae1dfa6b".to_string(),
            size: 104103,
            ..Default::default()
        }],
        hash_algo: lfs::LFS_HASH_ALGO.to_string(),
    };
    let lfs_client = LFSClient::from_url(&Url::parse(PUBLIC_FIXTURE_REPO).unwrap());
    let response = lfs_client
        .client
        .post(lfs_client.batch_url.clone())
        .json(&batch_request)
        .headers(lfs::LFS_HEADERS.clone())
        .send()
        .await
        .expect("LFS batch request should complete");
    let text = response
        .text()
        .await
        .expect("LFS response body is readable");
    let _response: LfsBatchResponse =
        serde_json::from_str(&text).expect("LFS batch response should be JSON");
}
