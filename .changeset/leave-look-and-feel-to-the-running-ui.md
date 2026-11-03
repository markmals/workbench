---
"workbench": minor
---

Proposals now say what a feature does and leave its look and feel — layout, sizes, spacing, type, color, motion, which control presents a choice, how it is operated, and wording — to the human on the running UI. Look-and-feel feedback during human review is applied while the proposal stays `active-review`, with no proposal edit, status change, readiness report, or second adversarial review. Polishing an interface between proposals needs no proposal question; the new `polishing-a-ui` skill describes the loop and stops when a request would change what the feature does.

What adopters have to do: copy `.agents/skills/polishing-a-ui/`, and merge the changes to `AGENTS.md` ("Does this change need a proposal?" and "Stay inside the proposal"), `PROCESS.md` (the new Look and feel section, the self-check, and Human Review), `README.md`, the `writing-a-proposal`, `implementing-a-proposal`, `reviewing-an-implementation`, and `adversarial-review` skills, the proposal and pull-request templates, the status messages in `tools/status.ts`, and the `running-a-feature` and `writing-a-good-proposal` guides. Existing proposals need no edits; look-and-feel details already written into one stand as decided at the time and may be refined through polish without revising it.
