-- HM-01: extend Repository agent_session.agent_kind CHECK with 'hermes'.
--
-- The then-complete set is the eight existing kinds, including grok and
-- opencode. `pi` stays out of the CHECK; PI-01 has not added that identity.
-- The runner applies this on a dedicated connection (ADR-CAP-CHECK):
-- foreign_keys=OFF, claim-first receipt, create-copy-drop-rename, then
-- foreign_key_check and integrity_check. This body must not run inside a
-- pooled transaction. A direct execute aborts instead of dropping the table.
SELECT RAISE(ABORT, 'agent_kind hermes CHECK expansion requires the dedicated migration connection');
