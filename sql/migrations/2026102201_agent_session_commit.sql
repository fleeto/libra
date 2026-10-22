-- CX-11: repository-local links from captured sessions to code commits.
-- OID format is explicit because repositories may use SHA-1, SHA-256 or BLAKE3.
-- The composite key permits many sessions on one commit and is retry-safe.
CREATE TABLE IF NOT EXISTS `agent_session_commit` (
    `session_id` TEXT NOT NULL REFERENCES `agent_session`(`session_id`) ON DELETE CASCADE,
    `hash_kind` TEXT NOT NULL CHECK (`hash_kind` IN ('sha1', 'sha256', 'blake3')),
    `commit_oid` TEXT NOT NULL CHECK (
        `commit_oid` NOT GLOB '*[^0-9a-f]*' AND
        ((`hash_kind` = 'sha1' AND length(`commit_oid`) = 40) OR
         (`hash_kind` IN ('sha256', 'blake3') AND length(`commit_oid`) = 64))
    ),
    PRIMARY KEY (`session_id`, `hash_kind`, `commit_oid`)
);
CREATE INDEX IF NOT EXISTS `idx_agent_session_commit_oid`
    ON `agent_session_commit` (`hash_kind`, `commit_oid`, `session_id`);
