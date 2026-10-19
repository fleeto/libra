-- GK-01 down: restore the previous agent_kind CHECK only when no grok row exists.
--
-- The runner performs the rebuild on the dedicated connection. If any
-- agent_session row uses agent_kind='grok', the rollback fails and keeps
-- those rows and this receipt. A direct execute aborts.
SELECT RAISE(ABORT, 'agent_kind grok CHECK rollback requires the dedicated migration connection');
