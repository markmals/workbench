# Templates

| Template | Destination path | Copied by |
| --- | --- | --- |
| `VISION.md` | `VISION.md` | `completing-a-feature` |
| `PROPOSAL.md` | `proposals/<NNNN>-<slug>.md` | `writing-a-proposal` |
| `POLICY.md` | `policies/<NNNN>-<slug>.md` | `implementing-a-proposal`, `completing-a-feature` |
| `DECISION.md` | `decisions/<NNNN>-<slug>.md` | `implementing-a-proposal`, `completing-a-feature` |
| `GUIDE.md` | `guides/<slug>.md` | `implementing-a-proposal` |
| `FEATURE.feature` | `specs/features/<slug>.feature`, or the project's existing location | `specifying-behavior-with-gherkin` |
| `READINESS_REPORT.md` | pull-request comment | `reviewing-an-implementation` |

`FEATURE.feature` is optional. Copy it only for a surface the project specifies in Gherkin; API contracts, schemas, and reference prose need no template.
