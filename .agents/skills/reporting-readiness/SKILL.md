---
name: reporting-readiness
description: Use when Review is complete and a draft pull request needs an evidence-backed readiness comment. Do not use as a substitute for human review.
---

# Reporting Readiness

**Phase 4, stage 4.** Compose `.agents/templates/READINESS_REPORT.md`, post it as a PR comment, then mark the PR ready for review. The report is the agent's durable statement; write only what this session's evidence supports.

## Compose the report

Fill every template section. Under `## Proposal`, state the proposal status:

| State                 | Required statement                                                      |
| --------------------- | ----------------------------------------------------------------------- |
| Fully implemented     | Every promised behavior is implemented.                                 |
| Partially implemented | Name the completed behavior, omitted behavior, limitations, and reason. |

Partial implementation is allowed. Silent partial implementation is not.

Include these evidence-backed sections:

| Section                         | Include                                                                                                                                         |
| ------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| Cross-artifact review           | Inventory verdict, divergences, and their resolution.                                                                                           |
| Adversarial review              | Report verdict and every declined finding in its own section, with reasoning. Use `- None.` when none were declined.                            |
| Validation                      | Each command actually run in this session and its observed result. Use `verification-before-completion`; never report an unrun gate as passing. |
| Preview                         | Link and instructions for what to try. Use `producing-a-preview`; never post a bare URL.                                                        |
| Defects discovered and deferred | Every deliberately deferred, out-of-scope defect as a linked GitHub issue. Use `tracking-defects`; write `- None.` when none exist.             |

The adversarial stage loses its value if the primary agent quietly drops a finding it disputes. Preserve declined findings even when they are narrow, uncomfortable, or rejected.

## Post as a service account

Each project must configure a service-account credential before this step: a bot token or GitHub App installation token in the environment. The kit does not ship a credential.

With a configured service account:

```sh
GH_TOKEN="$SERVICE_ACCOUNT_TOKEN" gh pr comment <pr-number> --body-file readiness-report.md
GH_TOKEN="$SERVICE_ACCOUNT_TOKEN" gh pr ready <pr-number>
```

Posting under that identity makes clear which statements the agent made. Confirm the authenticated identity before posting.

Without a service account, post from whatever authenticated identity is available and say so in the report: `Posted from <identity>; no service account is configured.` Do not imply bot authorship.

## Mark ready

After posting the report, mark the PR ready for review. Ready means the agent believes the work is human-reviewable; it does not claim the work is correct.

```sh
gh pr ready <pr-number>
```

## Red flags

- You called a proposal fully implemented while behavior is absent.
- You omitted a declined adversarial finding.
- You copied a previous validation result or claimed an unrun gate passed.
- You linked a preview without telling the human what to exercise.
- You deferred a defect without its issue link.
- You posted as a human while presenting the report as service-account output.
- You treated ready-for-review as human acceptance or correctness.

Any of these means correct the report before posting.

## Related skills

- `verification-before-completion` — fresh validation evidence.
- `producing-a-preview` — reviewable behavior and instructions.
- `tracking-defects` — durable records for legitimate deferred defects.
