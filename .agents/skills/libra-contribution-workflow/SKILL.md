---
name: libra-contribution-workflow
description: Guide contributors through Libra issue triage, plan and task discovery, dependency checks, card execution, validation, and PR preparation. Use whenever someone reports a Libra problem, asks how to contribute, wants to claim or start a task, asks about plans or cross-plan dependencies, or prepares a Libra change. Follow the current repository guidance as the source of truth.
---

# Libra Contribution Workflow

Help a contributor move from an initial report or idea to a well-scoped, validated contribution. Identify their current stage and whether they want guidance, planning, or implementation. Work in small steps and explain the next useful action.

## Read current guidance

Before giving repository-specific process instructions, inspect the current checkout and read the applicable sources:

- `AGENTS.md` for repository rules, command conventions, validation, and version-control workflow.
- `docs/contributing.md` for community contribution paths and PR expectations.
- `docs/development/plan/plan-template.md` for current plan and task-card requirements.
- `docs/development/plan/plan-status.md` for active plans, card states, deferred work, and cross-plan dependencies.
- The relevant plan, especially `docs/development/plan/issues/<number>.md` when work maps to a GitHub Issue.

Treat these files and current source, tests, and user docs as authoritative. Older plan examples are historical evidence, not policy. Do not invent labels, ownership rules, commands, dependencies, or acceptance criteria.

## Triage an Issue or idea

1. Restate the observed problem or desired outcome.
2. Check for expected and actual behavior, reproduction steps, environment, and user impact as applicable. Ask focused questions only when missing details change diagnosis or scope.
3. Classify provisionally as a bug, feature request, documentation issue, usage question, or unconfirmed report. Mark uncertain conclusions as hypotheses.
4. Search local plans, status records, source, tests, and docs for existing work before proposing a duplicate.
5. Recommend the next step: answer or gather evidence, link existing work, prepare a small direct change, or draft a plan and cards. Leave labels, assignment, priority, and project acceptance to maintainers.

Issue triage is read-only unless the user explicitly authorizes a remote action. Do not create, edit, label, assign, comment on, or close a GitHub Issue without explicit authorization for that action.

## Relate Issues, plans, cards, and dependencies

- An Issue records a problem or requested outcome. In this repository, `docs/development/plan/issues/<number>.md` commonly maps one-to-one to a GitHub Issue and serves as its executable plan. Dated plans may cover a broader initiative or multiple related requests.
- A plan describes outcome, scope, evidence, dependencies, and acceptance. It may have one or many cards; an Issue does not automatically equal one card.
- A task card is an independently actionable unit with a clear scope, dependencies, write set, deliverables, acceptance criteria, and verification commands, following the current template.
- In-plan dependencies reference concrete task IDs and form a directed acyclic graph. `A -> B` means A must precede B.
- Cross-plan, external-service, approval, and upstream-release dependencies must be registered as `DEP-*` before cards reference them. Check the artifact, owner, evidence, availability criterion, and failure or timeout path.
- A card with no in-plan prerequisites may still be blocked by review gates, an external `DEP-*`, an overlapping write set, or other repository constraints. Do not call it ready until all applicable gates are satisfied.
- Keep lifecycle and acceptance distinct. When advancing a card, update its plan and `plan-status.md` together, as required by the current template.
- Card completion does not automatically close an Issue; plan completion does not prove every related Issue is resolved. Summarize remaining work and leave remote Issue status to the user or maintainer.

## Prepare a plan or select a card

For new planned work:

1. Check `plan-status.md` and relevant plans for duplicates, conflicts, cards, and dependencies.
2. Ground the proposal in current source, tests, docs, and compatibility behavior; separate verified facts from assumptions.
3. Draft with the current template. Make cards independently executable and reviewable, with exact deliverables and verification.
4. Connect internal dependencies between concrete cards. Register cross-plan and external dependencies as `DEP-*`; state whether each blocks start or only acceptance.
5. Include tests, docs, compatibility, rollback, security, and release handling when required by the changed surface.
6. Present the draft or recommended existing card for maintainer review. Do not begin implementation while required review, dependencies, or decisions are pending.

For an existing card, report lifecycle, acceptance, prerequisite cards, `DEP-*` status, write-set conflicts, and exact start gates before recommending it to a newcomer.

## Branch and worktree context

This skill does not require contributors to use a particular branch, branch name, or base, and it does not require switching branches before implementation. Respect the branch the contributor is already using unless the contributor, task card, or current repository policy specifies a different branch operation.

Before edits, inspect `libra status --short --branch` and preserve existing work. If the worktree is dirty, identify unrelated changes and avoid mixing or overwriting them. If branch selection matters for the task but is unspecified, explain the relevant context and ask before creating or switching branches. When a branch operation is explicitly required, use Libra commands and verify its result; do not use raw Git for repository state in this Libra checkout. Read-only triage and investigation do not require a branch change.

## Execute and validate a card

Once the contributor selected the card and implementation is authorized:

1. Re-read the card and refresh source, test, and doc anchors against the current checkout.
2. Follow the branch and worktree context above before the first edit.
3. Follow the card's scope and write set. If scope expands, dependencies change, or write sets overlap, update the plan or resolve sequencing first.
4. Add or update focused tests and user documentation as required by the card and repository rules.
5. Run the exact verification required by the card and current repository guidance. Follow current ER-13/ER-14 rules; do not copy obsolete commands from old plans.
6. Report actual results. Do not mark a card complete while acceptance, release, or remote gates remain pending.
7. Update the card and `plan-status.md` in the same change when lifecycle, acceptance, dependency, or deferred status changes.

## PR and GitHub interactions

Prepare a concise PR description with intent, related Issue(s), changes, tests actually run, and reproduction or sample output for user-visible behavior. Follow current DCO/signing and Libra commit/push instructions in `AGENTS.md`.

Use `gh` only when an explicitly authorized step needs live GitHub interaction, such as viewing or creating a PR or a plan-mandated release. Check that `gh` is installed and authenticated before relying on it. If unavailable, continue local planning and validation and provide the manual next step. Do not create/post Issue comments, assign or label Issues, close Issues, merge, push, tag, or publish a release without explicit authorization for that operation.

After review feedback, map requested changes to the relevant card and acceptance criteria. Keep local card/plan status separate from GitHub Issue/PR status and report unsynchronized state.

## Response format

At each step, state the current stage and evidence, the recommended next action and unmet prerequisites, relevant identifiers/links, and which actions are local versus external. Briefly explain terms such as `DEP-*`, lifecycle, acceptance, focused tests, and plan closeout when first used. Prefer one useful next action over a generic checklist.
