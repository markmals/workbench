---
title: CLI Reference
---

# CLI Reference

`workbench` is the Workbench package's CLI. Agents run it directly; humans usually reach it through the project's `mise run status`, `mise run validate`, and `mise run beads:*` tasks. This page is generated from the command registry by `mise run cli:reference`; do not edit it by hand.

Every command follows one contract:

- **Results** are JSON on stdout: compact when stdout is not a terminal, indented when it is. `--output text` renders the same result for humans. Progress, warnings, and diagnostics go to stderr.
- **Collections** return `items`, plus `hasMore` and `nextCursor` whenever the command bounds the result; pass `nextCursor` back as `--cursor` for the next page.
- **Modifying commands** accept `--dry-run`, which validates the input, resolves the target, and describes the changes without making them. Their receipts carry a `status`, the target, and the identifiers they produced.
- **Failures** write `{"error": {"code", "message", "retryable", "details", "suggestions"}}` to stderr and exit with the code's status. A check that ran and found violations still writes its result to stdout.
- **No command prompts.** Missing input fails at once with `INVALID_ARGUMENT`.

Discover a command with `workbench cli search "<task>"`, inspect it with `workbench schema <resource> <operation>`, preview it with `--dry-run`, then run it and verify the receipt.

## Options every command accepts

| Argument | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `--output <choice>` | `json`, `text` | no | `json` | Result format: JSON (compact when piped, indented on a terminal) or text for humans |

## Error codes

| Code | Exit | Retryable | Meaning |
| --- | --- | --- | --- |
| `CHECK_FAILED` | 1 | No | A check ran and found violations. |
| `INVALID_ARGUMENT` | 2 | No | Input is missing or malformed. |
| `NOT_FOUND` | 3 | No | A named record, item, commit, or file does not exist. |
| `PRECONDITION_FAILED` | 3 | No | The project is not ready for the operation. |
| `DEPENDENCY_MISSING` | 3 | No | A required program (git, gh, bd) is absent. |
| `UNAUTHENTICATED` | 3 | No | gh or Git credentials are missing or rejected. |
| `CONFLICT` | 4 | No | Another claimant holds the item, a claim was lost, or a Beads sync stopped on a conflict. |
| `TIMEOUT` | 5 | Yes | An operation exceeded its time limit. |
| `UNAVAILABLE` | 5 | Yes | The remote or the network was unreachable. |
| `INTERNAL` | 70 | No | A defect in workbench itself. |

## Commands

### `cli`

Discover workbench's own commands.

#### `workbench cli search`

Find the workbench command for a task described in plain words.

Ranks every workbench command against the query by its name, summary, description, options, and declared synonyms, and returns the best matches with the exact command to run. Runs locally, needs no credentials, and never changes anything. Inspect a match with `workbench schema <command>` before running it.

- **Effect:** reads only
- **Requires:** nothing beyond `workbench`
- **Dry run:** no
- **Idempotent:** yes
- **Errors:** `INVALID_ARGUMENT`, `INTERNAL`

| Argument | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `<query>` | string, repeatable | yes | — | What you want to do, in plain words; several words need no quotes |
| `--limit <integer>` | integer | no | `5` | The most matches to return |
| `--cursor <string>` | string | no | — | Resume after a previous page: the nextCursor it returned |

Examples:

```sh
# Find how to claim a Beads item
workbench cli search 'claim a work item'
# Find how to add a release note
workbench cli search 'add a change file' --limit 3
```

Result schema:

```json
{
  "$defs": {
    "CommandMatch": {
      "description": "One command that fits the query.",
      "properties": {
        "command": {
          "description": "The exact command to run, without its arguments.",
          "type": "string"
        },
        "effect": {
          "description": "`reads`, or `modifies <what>`.",
          "type": "string"
        },
        "id": {
          "description": "The registry identifier, such as `beads.claim`.",
          "type": "string"
        },
        "score": {
          "description": "Relevance from 0 to 1.",
          "format": "double",
          "type": "number"
        },
        "summary": {
          "type": "string"
        }
      },
      "required": [
        "command",
        "id",
        "summary",
        "effect",
        "score"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "One page of a ranked collection.",
  "properties": {
    "hasMore": {
      "description": "Whether more matches follow this page.",
      "type": "boolean"
    },
    "items": {
      "items": {
        "$ref": "#/$defs/CommandMatch"
      },
      "type": "array"
    },
    "nextCursor": {
      "description": "Pass as `--cursor` to get the next page; null when there is none.",
      "type": [
        "string",
        "null"
      ]
    }
  },
  "required": [
    "items",
    "hasMore"
  ],
  "title": "Page",
  "type": "object"
}
```

#### `workbench cli completions`

Print a shell completion script for workbench.

Generates the completion script for the named shell from the command registry. The JSON result holds the script in `script`; `--output text` prints the script alone, ready to redirect into the shell's completion directory.

- **Effect:** reads only
- **Requires:** nothing beyond `workbench`
- **Dry run:** no
- **Idempotent:** yes
- **Errors:** `INVALID_ARGUMENT`, `INTERNAL`

| Argument | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `<shell>` | `bash`, `zsh`, `fish`, `elvish`, `powershell` | yes | — | The shell to generate completions for |

Examples:

```sh
# Completions for zsh
workbench cli completions zsh
```

Result schema:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "properties": {
    "script": {
      "description": "The completion script.",
      "type": "string"
    },
    "shell": {
      "type": "string"
    }
  },
  "required": [
    "shell",
    "script"
  ],
  "title": "Completions",
  "type": "object"
}
```

### `schema`

Inspect a command's full specification.

#### `workbench schema`

Show a command's arguments, defaults, effects, errors, examples, and output schema.

With no command, lists every command with its summary and effect. With a command path such as `beads claim` (or its identifier, `beads.claim`), returns that command's full registry entry: each argument with its type, whether it is required, its default, and its accepted values; whether it reads or modifies state; what it requires; whether it supports --dry-run and is idempotent; the error codes it can return; examples; and the JSON Schema of its result. Never changes anything.

- **Effect:** reads only
- **Requires:** nothing beyond `workbench`
- **Dry run:** no
- **Idempotent:** yes
- **Errors:** `NOT_FOUND`, `INTERNAL`

| Argument | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `<command>` | string, repeatable | no | — | The command path, such as `beads claim`; omit to list every command |

Examples:

```sh
# List every command
workbench schema
# Inspect the command search
workbench schema cli search
```

Result schema:

```json
{
  "$defs": {
    "ArgumentDescription": {
      "properties": {
        "acceptedValues": {
          "description": "The only values a `choice` accepts.",
          "items": {
            "type": "string"
          },
          "type": [
            "array",
            "null"
          ]
        },
        "default": {
          "type": [
            "string",
            "null"
          ]
        },
        "kind": {
          "description": "`positional`, `option`, or `flag`.",
          "type": "string"
        },
        "multiple": {
          "description": "Whether it accepts several values.",
          "type": "boolean"
        },
        "name": {
          "type": "string"
        },
        "required": {
          "type": "boolean"
        },
        "summary": {
          "type": "string"
        },
        "type": {
          "description": "`string`, `path`, `integer`, `text` (text, `@<file>`, or `-` for stdin), `choice`, or `boolean`.",
          "type": "string"
        },
        "usage": {
          "description": "How to write it: `<query>`, `--limit <integer>`, `--dry-run`.",
          "type": "string"
        }
      },
      "required": [
        "name",
        "kind",
        "usage",
        "type",
        "required",
        "multiple",
        "summary"
      ],
      "type": "object"
    },
    "CommandDescription": {
      "description": "A command's full specification, as `workbench schema <command>` returns it.",
      "properties": {
        "arguments": {
          "description": "The command's own arguments, then the options every command accepts.",
          "items": {
            "$ref": "#/$defs/ArgumentDescription"
          },
          "type": "array"
        },
        "command": {
          "description": "The command to run, without arguments.",
          "type": "string"
        },
        "description": {
          "type": "string"
        },
        "dryRun": {
          "description": "Whether the command accepts `--dry-run`.",
          "type": "boolean"
        },
        "effect": {
          "$ref": "#/$defs/EffectDescription"
        },
        "errors": {
          "description": "Every error the command can report on stderr.",
          "items": {
            "$ref": "#/$defs/ErrorDescription"
          },
          "type": "array"
        },
        "examples": {
          "items": {
            "$ref": "#/$defs/ExampleDescription"
          },
          "type": "array"
        },
        "id": {
          "description": "Stable identifier: the command path joined with dots.",
          "type": "string"
        },
        "idempotent": {
          "description": "Whether running it twice with the same input leaves the same state as running it once.",
          "type": "boolean"
        },
        "output": {
          "description": "JSON Schema of the result the command writes to stdout."
        },
        "requires": {
          "description": "What the command needs besides workbench: `git`, `gh`, `gh-authentication`, `bd`,\n`network`, or `workbench-link` (the project's `.workbench` package link).",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "summary": {
          "type": "string"
        }
      },
      "required": [
        "id",
        "command",
        "summary",
        "description",
        "arguments",
        "effect",
        "requires",
        "dryRun",
        "idempotent",
        "errors",
        "examples",
        "output"
      ],
      "type": "object"
    },
    "CommandList": {
      "properties": {
        "items": {
          "items": {
            "$ref": "#/$defs/CommandSummary"
          },
          "type": "array"
        }
      },
      "required": [
        "items"
      ],
      "type": "object"
    },
    "CommandSummary": {
      "properties": {
        "command": {
          "type": "string"
        },
        "effect": {
          "description": "`reads`, or `modifies <what>`.",
          "type": "string"
        },
        "id": {
          "type": "string"
        },
        "summary": {
          "type": "string"
        }
      },
      "required": [
        "command",
        "id",
        "summary",
        "effect"
      ],
      "type": "object"
    },
    "EffectDescription": {
      "properties": {
        "kind": {
          "description": "`reads` or `modifies`.",
          "type": "string"
        },
        "what": {
          "description": "What a modifying command changes.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "kind"
      ],
      "type": "object"
    },
    "ErrorDescription": {
      "properties": {
        "code": {
          "type": "string"
        },
        "exit": {
          "description": "The exit status the error sets.",
          "format": "int32",
          "type": "integer"
        },
        "meaning": {
          "type": "string"
        },
        "retryable": {
          "description": "Whether retrying the same command can succeed.",
          "type": "boolean"
        }
      },
      "required": [
        "code",
        "exit",
        "retryable",
        "meaning"
      ],
      "type": "object"
    },
    "ExampleDescription": {
      "properties": {
        "command": {
          "type": "string"
        },
        "summary": {
          "type": "string"
        }
      },
      "required": [
        "summary",
        "command"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "anyOf": [
    {
      "$ref": "#/$defs/CommandList",
      "description": "With no command: every command."
    },
    {
      "$ref": "#/$defs/CommandDescription",
      "description": "With a command: its full specification."
    }
  ],
  "title": "SchemaResult"
}
```

### `docs`

Search the Workbench package's documentation.

#### `workbench docs search`

Search the Workbench package's contract, rules, skills, templates, and CLI reference.

Searches the Markdown in the project's Workbench package (the `.workbench` link) and returns the best-matching files with a `path` under `.workbench/`, a `title`, and a `snippet` around the best match. Read the file at `path` for detail. Runs offline and never changes anything. Fails with PRECONDITION_FAILED when `.workbench` is missing; `mise install` restores it.

- **Effect:** reads only
- **Requires:** `git`, `workbench-link`
- **Dry run:** no
- **Idempotent:** yes
- **Errors:** `INVALID_ARGUMENT`, `PRECONDITION_FAILED`, `DEPENDENCY_MISSING`, `INTERNAL`

| Argument | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `<query>` | string, repeatable | yes | — | The topic, in plain words; several words need no quotes |
| `--limit <integer>` | integer | no | `5` | The most matches to return |
| `--cursor <string>` | string | no | — | Resume after a previous page: the nextCursor it returned |

Examples:

```sh
# Find guidance on writing a proposal
workbench docs search 'write a proposal'
```

Result schema:

```json
{
  "$defs": {
    "DocumentMatch": {
      "description": "One package file that matches the query.",
      "properties": {
        "path": {
          "description": "The file to read, relative to the project root: `.workbench/skills/<name>/SKILL.md`.",
          "type": "string"
        },
        "score": {
          "description": "Relevance from 0 to 1.",
          "format": "double",
          "type": "number"
        },
        "snippet": {
          "description": "The line that best matches the query.",
          "type": "string"
        },
        "title": {
          "type": "string"
        }
      },
      "required": [
        "path",
        "title",
        "snippet",
        "score"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "One page of a ranked collection.",
  "properties": {
    "hasMore": {
      "description": "Whether more matches follow this page.",
      "type": "boolean"
    },
    "items": {
      "items": {
        "$ref": "#/$defs/DocumentMatch"
      },
      "type": "array"
    },
    "nextCursor": {
      "description": "Pass as `--cursor` to get the next page; null when there is none.",
      "type": [
        "string",
        "null"
      ]
    }
  },
  "required": [
    "items",
    "hasMore"
  ],
  "title": "Page",
  "type": "object"
}
```

### `records`

The project's proposals, policies, decisions, guides, and vision.

#### `workbench records validate`

Check the frontmatter of the project's proposals, policies, decisions, guides, and vision.

Checks the records at the project root, the Git top level of the current directory: `VISION.md` and the Markdown files directly inside `proposals/`, `policies/`, `decisions/`, and `guides/`. Directory READMEs are indexes and are skipped; absent or empty record directories are allowed; a Markdown file nested deeper is a violation; symlinked files are checked and symlinked directories are not traversed. Each record must have parsable frontmatter with its kind's required keys and an allowed status, an ID that matches its filename and is unique, and `supersedes` and `established-by` references that resolve. A proposal another supersedes must have status `superseded`, supersession must not form a cycle, and a proposal past drafting must not keep an unresolved clarification marker in its prose. The result lists every violation with its `path` relative to the project root and a `message`, and their `count`. With any violation the result still goes to stdout and the command fails with CHECK_FAILED (exit 1). Never changes anything. Fails with PRECONDITION_FAILED outside a Git checkout or when a record cannot be read, and with DEPENDENCY_MISSING when git is not installed.

- **Effect:** reads only
- **Requires:** `git`
- **Dry run:** no
- **Idempotent:** yes
- **Errors:** `CHECK_FAILED`, `PRECONDITION_FAILED`, `DEPENDENCY_MISSING`, `INTERNAL`

Examples:

```sh
# Check every record in the project
workbench records validate
```

Result schema:

```json
{
  "$defs": {
    "Violation": {
      "description": "One problem in one record file.",
      "properties": {
        "message": {
          "type": "string"
        },
        "path": {
          "description": "The file, relative to the project root.",
          "type": "string"
        }
      },
      "required": [
        "path",
        "message"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "Every problem the validator found in the project's records.",
  "properties": {
    "count": {
      "description": "How many violations there are; zero means the records are valid.",
      "format": "uint",
      "minimum": 0,
      "type": "integer"
    },
    "violations": {
      "description": "Every violation, in the order the validator found them.",
      "items": {
        "$ref": "#/$defs/Violation"
      },
      "type": "array"
    }
  },
  "required": [
    "violations",
    "count"
  ],
  "title": "ValidationReport",
  "type": "object"
}
```

### `commits`

Commit subjects and their Scoped Commits scopes.

#### `workbench commits check`

Check commit subjects against Scoped Commits: one message file, or every commit in a range.

Checks that each commit subject is a Scoped Commit, `<scope>: <imperative description>`, naming an accepted scope. Give exactly one form. `--message-file <path>` checks the message a commit-msg hook is about to commit: its subject is the first line that is neither blank nor a `#` comment, and a message with no such line passes. `<base> <head>` checks the subject of every commit reachable from head and not from base, and reports every bad subject, not only the first. Accepted scopes are the built-in process scopes (agents, ci, decision, deps, docs, guides, policy, proposal, release, rules, skills, templates, treewide, vision); the scopes listed one per line in the optional `.agents/commit-scopes` (blank lines and `#` comments ignored); and `<NNNN>-<slug>` for every `proposals/<NNNN>-<slug>.md`. Merge, revert, `fixup!`, and `squash!` subjects are exempt; Conventional Commits `type(scope):` subjects are rejected by name. The result lists each subject with its verdict, the violations, and every accepted scope. Any violation fails with CHECK_FAILED after the result is written. A missing message file or an unknown revision is NOT_FOUND; both forms, neither, or a base without a head is INVALID_ARGUMENT.

- **Effect:** reads only
- **Requires:** `git`
- **Dry run:** no
- **Idempotent:** yes
- **Errors:** `CHECK_FAILED`, `INVALID_ARGUMENT`, `NOT_FOUND`, `PRECONDITION_FAILED`, `DEPENDENCY_MISSING`, `INTERNAL`

| Argument | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `<base>` | string | no | — | The range's base: a commit, branch, or tag the checked commits are not reachable from |
| `<head>` | string | no | — | The range's head: the last commit to check |
| `--message-file <path>` | path | no | — | A commit message to check instead of a range, such as the commit-msg hook's file |

Examples:

```sh
# Check the message a commit-msg hook is about to commit
workbench commits check --message-file .git/COMMIT_EDITMSG
# Check every commit a branch adds to main
workbench commits check main feature
```

Result schema:

```json
{
  "$defs": {
    "CheckedSubject": {
      "properties": {
        "commit": {
          "description": "The full commit hash; absent for a message file.",
          "type": [
            "string",
            "null"
          ]
        },
        "scope": {
          "description": "The scope a well-formed subject names; absent otherwise.",
          "type": [
            "string",
            "null"
          ]
        },
        "subject": {
          "description": "Absent when the message holds only blank lines and `#` comments.",
          "type": [
            "string",
            "null"
          ]
        },
        "verdict": {
          "$ref": "#/$defs/Verdict"
        }
      },
      "required": [
        "verdict"
      ],
      "type": "object"
    },
    "RangeArguments": {
      "properties": {
        "base": {
          "type": "string"
        },
        "head": {
          "type": "string"
        }
      },
      "required": [
        "base",
        "head"
      ],
      "type": "object"
    },
    "Rule": {
      "description": "The rule a rejected subject breaks.",
      "oneOf": [
        {
          "const": "conventionalCommit",
          "description": "A Conventional Commits `type(scope):` subject.",
          "type": "string"
        },
        {
          "const": "unscoped",
          "description": "No `<scope>: ` prefix.",
          "type": "string"
        },
        {
          "const": "unknownScope",
          "description": "A scope that is not accepted.",
          "type": "string"
        }
      ]
    },
    "Verdict": {
      "description": "`accepted`; `exempt` for merge, revert, `fixup!`, and `squash!` subjects; `rejected`; or\n`noSubject` for a message with nothing to check.",
      "enum": [
        "accepted",
        "exempt",
        "rejected",
        "noSubject"
      ],
      "type": "string"
    },
    "Violation": {
      "properties": {
        "commit": {
          "description": "The full commit hash; absent for a message file.",
          "type": [
            "string",
            "null"
          ]
        },
        "message": {
          "description": "What is wrong; an unknown scope's message lists every accepted scope.",
          "type": "string"
        },
        "rule": {
          "$ref": "#/$defs/Rule",
          "description": "`conventionalCommit`, `unscoped`, or `unknownScope`."
        },
        "subject": {
          "type": "string"
        }
      },
      "required": [
        "subject",
        "rule",
        "message"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "properties": {
    "acceptedScopes": {
      "description": "Every scope a subject may use: the built-in process scopes, then those in\n`.agents/commit-scopes`, then `<NNNN>-<slug>` for each `proposals/<NNNN>-<slug>.md`.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "commits": {
      "description": "Every subject checked, oldest commit first.",
      "items": {
        "$ref": "#/$defs/CheckedSubject"
      },
      "type": "array"
    },
    "messageFile": {
      "description": "The commit-message file checked, as given; absent when a range was checked.",
      "type": [
        "string",
        "null"
      ]
    },
    "project": {
      "description": "The project whose scopes applied: the Git top level of the current directory.",
      "type": "string"
    },
    "range": {
      "anyOf": [
        {
          "$ref": "#/$defs/RangeArguments"
        },
        {
          "type": "null"
        }
      ],
      "description": "The range checked, as given; absent when a message file was checked."
    },
    "violations": {
      "description": "The subjects that break Scoped Commits, in the same order.",
      "items": {
        "$ref": "#/$defs/Violation"
      },
      "type": "array"
    }
  },
  "required": [
    "project",
    "commits",
    "violations",
    "acceptedScopes"
  ],
  "title": "CommitsCheck",
  "type": "object"
}
```

### `change-files`

Knope change files for release notes.

#### `workbench change-files create`

Write a Knope change file describing a change for the release notes.

Writes `.changeset/<name>.md`, the change file Knope turns into a version bump and a changelog entry. Write it for someone reading release notes, not for the reviewer of the diff. `<name>` comes from the summary the way Knope names change files: ASCII letters and digits lowercased, each run of spaces, `-`, and `_` one `_`, everything else dropped. The file is the frontmatter `<package>: <type>`, the heading `# <summary>`, then the body when given. `--type` must be `major`, `minor`, `patch`, or a type the package's `extra_changelog_sections` add in knope.toml. A single `[package]` is `default`; when knope.toml declares several `[packages.<name>]`, `--package` must name one. Invalid input is INVALID_ARGUMENT, with the accepted values in `details`; without knope.toml it is PRECONDITION_FAILED. It never overwrites: a summary whose file exists is INVALID_ARGUMENT. The receipt gives `status` `created`, the `project`, the file `path`, `type`, and `package`. `--dry-run` validates the same way, writes nothing, and returns `status` `dry-run` with the `content` it would write. Never prompts; use it instead of `knope document-change`.

- **Effect:** modifies one new file under .changeset/
- **Requires:** `git`
- **Dry run:** yes
- **Idempotent:** no
- **Errors:** `INVALID_ARGUMENT`, `PRECONDITION_FAILED`, `DEPENDENCY_MISSING`, `INTERNAL`

| Argument | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `--type <string>` | string | yes | — | The change type: major, minor, patch, or one the package's knope.toml sections add |
| `--summary <string>` | string | yes | — | One line for the changelog; it becomes the heading and names the file |
| `--package <string>` | string | no | — | The package the change affects; required when knope.toml declares several |
| `--body <text>` | text | no | — | Detail below the heading: text, @<file>, or - for stdin |
| `--dry-run` | boolean | no | — | Validate input, resolve the target, and describe the changes without making them |

Examples:

```sh
# Describe a new feature
workbench change-files create --type minor --summary 'Export reports as CSV'
# Preview a fix with detail, writing nothing
workbench change-files create --type patch --summary 'Keep the last row when exporting' --body 'Exports no longer drop the final row of a report.' --dry-run
```

Result schema:

```json
{
  "$defs": {
    "Status": {
      "enum": [
        "created",
        "dry-run"
      ],
      "type": "string"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `change-files create` wrote, or would write.",
  "properties": {
    "cannotPredict": {
      "description": "What a dry run cannot predict; empty when it predicts everything.",
      "items": {
        "type": "string"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "content": {
      "description": "The file a dry run would write.",
      "type": [
        "string",
        "null"
      ]
    },
    "package": {
      "description": "The package the change affects: `default` in a single-package project.",
      "type": "string"
    },
    "path": {
      "description": "The change file, relative to the project root: `.changeset/<name>.md`.",
      "type": "string"
    },
    "project": {
      "description": "The project root.",
      "type": "string"
    },
    "status": {
      "$ref": "#/$defs/Status",
      "description": "`created`, or `dry-run` when nothing was written."
    },
    "type": {
      "description": "The change type: `major`, `minor`, `patch`, or one the package's knope.toml sections add.",
      "type": "string"
    }
  },
  "required": [
    "status",
    "project",
    "path",
    "type",
    "package"
  ],
  "title": "ChangeFileReceipt",
  "type": "object"
}
```

#### `workbench change-files check`

Check that a commit range adds a valid change file or carries Release-Note: none.

Passes when the range, the commits reachable from head and not from base, either adds a Markdown file directly under `.changeset/` with valid change-file frontmatter, or has a commit carrying the Git trailer `Release-Note: none` (the key in any case, the value exactly `none`, parsed by Git the way `git interpret-trailers --parse` parses it). Added files are the ones head adds since its merge base with base, read as committed at head. Valid frontmatter is a `---` line, one `<package>: <type>` line per affected package, and a closing `---` line; when the project has a knope.toml, every package must be one it declares (`default` for a single `[package]`) and every type `major`, `minor`, `patch`, or one that package's `extra_changelog_sections` add. The result lists each added change file with its problems and the commits carrying the trailer. Otherwise it fails with CHECK_FAILED naming both ways to satisfy it. When the check passes but another added file is invalid, stderr warns about it, because Knope fails on it. An unknown revision is NOT_FOUND; base and head without shared history is INVALID_ARGUMENT; an unusable knope.toml is PRECONDITION_FAILED. CI runs it on every pull request except Knope's release pull request.

- **Effect:** reads only
- **Requires:** `git`
- **Dry run:** no
- **Idempotent:** yes
- **Errors:** `CHECK_FAILED`, `INVALID_ARGUMENT`, `NOT_FOUND`, `PRECONDITION_FAILED`, `DEPENDENCY_MISSING`, `INTERNAL`

| Argument | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `<base>` | string | yes | — | The range's base, such as the pull request's target branch |
| `<head>` | string | yes | — | The range's head, such as the pull request's last commit |

Examples:

```sh
# Check that a branch describes its change for release notes
workbench change-files check main feature
```

Result schema:

```json
{
  "$defs": {
    "AddedChangeFile": {
      "properties": {
        "path": {
          "description": "Relative to the project root: `.changeset/<name>.md`.",
          "type": "string"
        },
        "problems": {
          "description": "Why it is not a valid change file; empty when it is one.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "valid": {
          "description": "Whether Knope accepts its frontmatter.",
          "type": "boolean"
        }
      },
      "required": [
        "path",
        "valid",
        "problems"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "properties": {
    "base": {
      "type": "string"
    },
    "changeFiles": {
      "description": "The Markdown files the range adds directly under `.changeset/`, as committed at head.",
      "items": {
        "$ref": "#/$defs/AddedChangeFile"
      },
      "type": "array"
    },
    "head": {
      "type": "string"
    },
    "knopeToml": {
      "description": "Whether the project has a knope.toml, so packages and change types were checked\nagainst it. Without one, any well-formed frontmatter counts.",
      "type": "boolean"
    },
    "project": {
      "description": "The project root.",
      "type": "string"
    },
    "releaseNoteNone": {
      "description": "The commits in the range, oldest first, that carry the trailer `Release-Note: none`.",
      "items": {
        "type": "string"
      },
      "type": "array"
    }
  },
  "required": [
    "project",
    "base",
    "head",
    "knopeToml",
    "changeFiles",
    "releaseNoteNone"
  ],
  "title": "ChangeFilesCheck",
  "type": "object"
}
```
