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

### `context`

Inspect the project, pin, link, harness, and Beads setup.

#### `workbench context show`

Show the project, branch, pull request, Workbench pin and link, harnesses, Beads, and site.

Reports the project's Workbench context without judging or changing it: the project root (the Git top level of the current directory), the branch (null when HEAD is detached), and the `origin` URL; the branch's pull request from `gh`, or `unknown` with the reason (no branch, no `origin`, `gh` missing or unauthenticated, no answer within the time limit, or several pull requests for the branch); the `github:markmals/workbench` version `mise.toml` pins, this binary's version, and where `.workbench` points with the package version there (in Workbench's own repository it links to the working tree's `plugin/` and nothing is pinned); whether `.claude/settings.json` and `.omp/config.yml` declare the package, and the `.claude/skills` and `.claude/rules` links to the project's own skills and rules; the Beads workspace, its Dolt remote as `bd` reports it, and the claimant claims and closes act as (`BEADS_ACTOR`, or `<login user>@<short host name>`); and the documentation site: `docs/hugo.toml` and the workflows that build it, with their host. A missing `gh` or `bd` makes the affected field `unknown` rather than failing. `gh` and `bd` together get `--timeout` seconds. Use `workbench project validate` to check the setup.

- **Effect:** reads only
- **Requires:** `git`
- **Dry run:** no
- **Idempotent:** yes
- **Errors:** `INVALID_ARGUMENT`, `PRECONDITION_FAILED`, `DEPENDENCY_MISSING`, `INTERNAL`

| Argument | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `--timeout <integer>` | integer | no | `10` | Seconds to wait for `gh` and `bd`; past it, what they would report is unknown |

Examples:

```sh
# Show the project's Workbench context
workbench context show
```

Result schema:

```json
{
  "$defs": {
    "Beads": {
      "properties": {
        "claimant": {
          "description": "Who `workbench beads claim` and `close` act as on this machine.",
          "type": "string"
        },
        "remote": {
          "$ref": "#/$defs/BeadsRemote",
          "description": "The workspace's Dolt remote, which Beads syncs with."
        },
        "workspace": {
          "description": "`.beads` when the project has a Beads workspace, otherwise null.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "remote",
        "claimant"
      ],
      "type": "object"
    },
    "BeadsRemote": {
      "oneOf": [
        {
          "description": "The URL of the workspace's `origin` Dolt remote.",
          "properties": {
            "status": {
              "const": "configured",
              "type": "string"
            },
            "url": {
              "type": "string"
            }
          },
          "required": [
            "status",
            "url"
          ],
          "type": "object"
        },
        {
          "description": "The workspace has no `origin` Dolt remote yet.",
          "properties": {
            "status": {
              "const": "none",
              "type": "string"
            }
          },
          "required": [
            "status"
          ],
          "type": "object"
        },
        {
          "properties": {
            "reason": {
              "type": "string"
            },
            "status": {
              "const": "unknown",
              "type": "string"
            }
          },
          "required": [
            "status",
            "reason"
          ],
          "type": "object"
        }
      ]
    },
    "Declaration": {
      "description": "Whether one harness's project configuration loads the package.",
      "properties": {
        "detail": {
          "description": "What the file lacks, or why it cannot be read; null when declared.",
          "type": [
            "string",
            "null"
          ]
        },
        "path": {
          "description": "The harness's project configuration file.",
          "type": "string"
        },
        "state": {
          "$ref": "#/$defs/DeclarationState"
        }
      },
      "required": [
        "path",
        "state"
      ],
      "type": "object"
    },
    "DeclarationState": {
      "oneOf": [
        {
          "enum": [
            "declared"
          ],
          "type": "string"
        },
        {
          "const": "missing",
          "description": "The file does not exist.",
          "type": "string"
        },
        {
          "const": "incomplete",
          "description": "The file exists without the declaration.",
          "type": "string"
        },
        {
          "const": "unreadable",
          "description": "The file cannot be read or parsed.",
          "type": "string"
        }
      ]
    },
    "Harnesses": {
      "properties": {
        "claudeCode": {
          "$ref": "#/$defs/Declaration",
          "description": "Claude Code's declaration of the package in `.claude/settings.json`."
        },
        "ohMyPi": {
          "$ref": "#/$defs/Declaration",
          "description": "Oh My Pi's declaration of the package in `.omp/config.yml`."
        },
        "projectLinks": {
          "description": "`.claude/skills` and `.claude/rules`, through which Claude Code loads the project's own\nskills and rules from `.agents/`.",
          "items": {
            "$ref": "#/$defs/ProjectLink"
          },
          "type": "array"
        }
      },
      "required": [
        "claudeCode",
        "ohMyPi",
        "projectLinks"
      ],
      "type": "object"
    },
    "Link": {
      "description": "The `.workbench` link to the installed package.",
      "properties": {
        "path": {
          "description": "Always `.workbench`.",
          "type": "string"
        },
        "state": {
          "$ref": "#/$defs/LinkState"
        },
        "target": {
          "description": "The directory the link resolves to; for a dangling link, its target as written.",
          "type": [
            "string",
            "null"
          ]
        },
        "version": {
          "description": "The package version at the target: `version` in its `.claude-plugin/plugin.json`.",
          "type": [
            "string",
            "null"
          ]
        },
        "workingTree": {
          "description": "True in Workbench's own repository, where `.workbench` links to the working tree's\n`plugin/` and `mise.toml` pins no release.",
          "type": "boolean"
        }
      },
      "required": [
        "path",
        "state",
        "workingTree"
      ],
      "type": "object"
    },
    "LinkState": {
      "oneOf": [
        {
          "const": "linked",
          "description": "It resolves to a directory.",
          "type": "string"
        },
        {
          "const": "missing",
          "description": "There is no `.workbench`.",
          "type": "string"
        },
        {
          "const": "dangling",
          "description": "It points at nothing.",
          "type": "string"
        }
      ]
    },
    "Pin": {
      "description": "What `mise.toml` pins for Workbench.",
      "oneOf": [
        {
          "description": "`mise.toml` pins this version of `github:markmals/workbench`.",
          "properties": {
            "status": {
              "const": "pinned",
              "type": "string"
            },
            "version": {
              "type": "string"
            }
          },
          "required": [
            "status",
            "version"
          ],
          "type": "object"
        },
        {
          "description": "`mise.toml` is absent or pins no Workbench version.",
          "properties": {
            "status": {
              "const": "unpinned",
              "type": "string"
            }
          },
          "required": [
            "status"
          ],
          "type": "object"
        },
        {
          "description": "`mise.toml` exists but is not valid TOML.",
          "properties": {
            "reason": {
              "type": "string"
            },
            "status": {
              "const": "unreadable",
              "type": "string"
            }
          },
          "required": [
            "status",
            "reason"
          ],
          "type": "object"
        }
      ]
    },
    "ProjectLink": {
      "properties": {
        "path": {
          "type": "string"
        },
        "target": {
          "description": "Where the link points, as written; null when there is no link at the path.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "path"
      ],
      "type": "object"
    },
    "PullRequest": {
      "description": "The branch's pull request.",
      "oneOf": [
        {
          "properties": {
            "draft": {
              "type": "boolean"
            },
            "number": {
              "format": "uint64",
              "minimum": 0,
              "type": "integer"
            },
            "state": {
              "description": "`OPEN`, `CLOSED`, or `MERGED`.",
              "type": "string"
            },
            "status": {
              "const": "found",
              "type": "string"
            },
            "title": {
              "type": "string"
            },
            "url": {
              "type": "string"
            }
          },
          "required": [
            "status",
            "number",
            "url",
            "title",
            "state",
            "draft"
          ],
          "type": "object"
        },
        {
          "description": "GitHub has no pull request for the branch.",
          "properties": {
            "status": {
              "const": "none",
              "type": "string"
            }
          },
          "required": [
            "status"
          ],
          "type": "object"
        },
        {
          "description": "The pull request could not be determined.",
          "properties": {
            "reason": {
              "type": "string"
            },
            "status": {
              "const": "unknown",
              "type": "string"
            }
          },
          "required": [
            "status",
            "reason"
          ],
          "type": "object"
        }
      ]
    },
    "Site": {
      "properties": {
        "config": {
          "description": "`docs/hugo.toml` when present, otherwise null.",
          "type": [
            "string",
            "null"
          ]
        },
        "workflows": {
          "description": "The GitHub workflows that build the site (they run `docs:build`).",
          "items": {
            "$ref": "#/$defs/SiteWorkflow"
          },
          "type": "array"
        }
      },
      "required": [
        "workflows"
      ],
      "type": "object"
    },
    "SiteHost": {
      "oneOf": [
        {
          "enum": [
            "github-pages",
            "cloudflare"
          ],
          "type": "string"
        },
        {
          "const": "unknown",
          "description": "The workflow builds the site but publishes it nowhere Workbench recognizes.",
          "type": "string"
        }
      ]
    },
    "SiteWorkflow": {
      "properties": {
        "host": {
          "$ref": "#/$defs/SiteHost"
        },
        "path": {
          "type": "string"
        }
      },
      "required": [
        "path",
        "host"
      ],
      "type": "object"
    },
    "Workbench": {
      "properties": {
        "binary": {
          "description": "The version of the `workbench` binary that ran this command.",
          "type": "string"
        },
        "link": {
          "$ref": "#/$defs/Link"
        },
        "pin": {
          "$ref": "#/$defs/Pin",
          "description": "The `github:markmals/workbench` version `mise.toml` pins."
        }
      },
      "required": [
        "pin",
        "binary",
        "link"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "The project's Workbench context.",
  "properties": {
    "beads": {
      "$ref": "#/$defs/Beads"
    },
    "branch": {
      "description": "The checked-out branch, or null when HEAD is detached.",
      "type": [
        "string",
        "null"
      ]
    },
    "harnesses": {
      "$ref": "#/$defs/Harnesses"
    },
    "origin": {
      "description": "The URL of the `origin` remote, or null when there is none.",
      "type": [
        "string",
        "null"
      ]
    },
    "project": {
      "description": "The project root: the Git top level of the current directory.",
      "type": "string"
    },
    "pullRequest": {
      "$ref": "#/$defs/PullRequest"
    },
    "site": {
      "$ref": "#/$defs/Site"
    },
    "workbench": {
      "$ref": "#/$defs/Workbench"
    }
  },
  "required": [
    "project",
    "pullRequest",
    "workbench",
    "harnesses",
    "beads",
    "site"
  ],
  "title": "Context",
  "type": "object"
}
```

### `workflow`

Where the work stands and what happens next.

#### `workbench workflow status`

Report where the work stands, what happens next and who does it, and this machine's Beads work.

Reads the branch, its pull request (through `gh`), and the active proposal, and reports the phase, the situation, the next action, whether the agent or the human (`you`) takes it, the skill that covers it, notices, and the `facts` they were derived from. A pull request `gh` cannot look up — `gh` missing, not authenticated, or not answering — is `unknown` with the reason, never `none`. Before reporting it runs one Beads sync with the project's remote, the same one `workbench beads sync` runs; `--no-sync` skips it. `beads` then holds the sync's `outcome` (`synced`, `skipped`, `failed`, `conflict`, or `blocked`) and `detail`, the ready work, the items in progress and who holds them, and the project's memories; or `unavailable` with the reason. `--timeout` limits the sync and each `gh` query (default 20 seconds). Beads outcomes never change the exit status: the command exits 0 whenever it can read the checkout.

- **Effect:** reads only
- **Requires:** `git`, `gh`, `gh-authentication`, `bd`, `network`
- **Dry run:** no
- **Idempotent:** yes
- **Errors:** `INVALID_ARGUMENT`, `PRECONDITION_FAILED`, `DEPENDENCY_MISSING`, `INTERNAL`

| Argument | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `--no-sync` | boolean | no | — | Report this machine's Beads replica without syncing it first |
| `--timeout <integer>` | integer | no | `20` | Seconds the Beads sync, and each GitHub query, may take |

Examples:

```sh
# Orient at the start of a session
workbench workflow status
# Report without syncing Beads, as when offline
workbench workflow status --no-sync
```

Result schema:

```json
{
  "$defs": {
    "Actor": {
      "description": "Who acts next: the agent, or the human reading the report.",
      "enum": [
        "agent",
        "you"
      ],
      "type": "string"
    },
    "BeadsReport": {
      "anyOf": [
        {
          "$ref": "#/$defs/BeadsState"
        },
        {
          "$ref": "#/$defs/BeadsUnavailable"
        }
      ],
      "description": "This machine's Beads replica after the sync, or why it cannot be read."
    },
    "BeadsState": {
      "properties": {
        "inProgress": {
          "description": "Items in progress and who holds them, from `bd list --status in_progress`.",
          "items": {
            "$ref": "#/$defs/InProgressItem"
          },
          "type": "array"
        },
        "memories": {
          "description": "The project's memories, from `bd prime --memories-only`.",
          "type": "string"
        },
        "ready": {
          "description": "Work no open item blocks, from `bd ready`.",
          "items": {
            "$ref": "#/$defs/ReadyItem"
          },
          "type": "array"
        },
        "sync": {
          "$ref": "#/$defs/SyncResult"
        }
      },
      "required": [
        "sync",
        "ready",
        "inProgress",
        "memories"
      ],
      "type": "object"
    },
    "BeadsUnavailable": {
      "properties": {
        "unavailable": {
          "description": "Why: `bd` is missing, there is no workspace or database, or a read after the sync failed.",
          "type": "string"
        }
      },
      "required": [
        "unavailable"
      ],
      "type": "object"
    },
    "Facts": {
      "properties": {
        "branch": {
          "description": "The checked-out branch, or `null` when Git cannot name it.",
          "type": [
            "string",
            "null"
          ]
        },
        "defaultBranch": {
          "description": "The repository's default branch, or `null` when it cannot be determined.",
          "type": [
            "string",
            "null"
          ]
        },
        "hasUpstream": {
          "type": "boolean"
        },
        "isDefaultBranch": {
          "description": "Whether the branch is the default branch; `null` when that is unknown.",
          "type": [
            "boolean",
            "null"
          ]
        },
        "proposal": {
          "anyOf": [
            {
              "$ref": "#/$defs/Proposal"
            },
            {
              "type": "null"
            }
          ],
          "description": "The proposal the work is about, if any."
        },
        "proposalSelection": {
          "$ref": "#/$defs/ProposalSelection"
        },
        "proposalSuccessor": {
          "anyOf": [
            {
              "$ref": "#/$defs/ProposalSummary"
            },
            {
              "type": "null"
            }
          ],
          "description": "The proposal that supersedes a superseded one."
        },
        "pullRequest": {
          "$ref": "#/$defs/PullRequestLookup"
        },
        "unpushedCommits": {
          "description": "Commits on the branch that its upstream does not have.",
          "format": "uint64",
          "minimum": 0,
          "type": "integer"
        }
      },
      "required": [
        "hasUpstream",
        "unpushedCommits",
        "pullRequest",
        "proposalSelection"
      ],
      "type": "object"
    },
    "InProgressItem": {
      "properties": {
        "assignee": {
          "type": [
            "string",
            "null"
          ]
        },
        "id": {
          "type": "string"
        },
        "title": {
          "type": "string"
        }
      },
      "required": [
        "id",
        "title"
      ],
      "type": "object"
    },
    "Notice": {
      "description": "Something the next actor should know besides the phase.",
      "oneOf": [
        {
          "description": "The proposal has unresolved clarification markers.",
          "properties": {
            "count": {
              "format": "uint",
              "minimum": 0,
              "type": "integer"
            },
            "kind": {
              "const": "clarifications",
              "type": "string"
            }
          },
          "required": [
            "kind",
            "count"
          ],
          "type": "object"
        },
        {
          "description": "The branch has commits its upstream does not.",
          "properties": {
            "count": {
              "format": "uint64",
              "minimum": 0,
              "type": "integer"
            },
            "kind": {
              "const": "unpushed-commits",
              "type": "string"
            }
          },
          "required": [
            "kind",
            "count"
          ],
          "type": "object"
        },
        {
          "description": "The proposal is in active review, but its pull request is still a draft.",
          "properties": {
            "kind": {
              "const": "draft-active-review",
              "type": "string"
            }
          },
          "required": [
            "kind"
          ],
          "type": "object"
        },
        {
          "description": "The proposal was chosen by a heuristic, or names another pull request.",
          "properties": {
            "kind": {
              "const": "ambiguous-proposal-selection",
              "type": "string"
            },
            "otherCandidates": {
              "items": {
                "$ref": "#/$defs/ProposalSummary"
              },
              "type": "array"
            },
            "proposalPullRequest": {
              "type": [
                "string",
                "null"
              ]
            },
            "pullRequestMismatch": {
              "type": "boolean"
            },
            "pullRequestNumber": {
              "format": "uint64",
              "minimum": 0,
              "type": [
                "integer",
                "null"
              ]
            },
            "selected": {
              "type": [
                "string",
                "null"
              ]
            }
          },
          "required": [
            "kind",
            "otherCandidates",
            "pullRequestMismatch"
          ],
          "type": "object"
        }
      ]
    },
    "Proposal": {
      "description": "A proposal as its frontmatter states it. A field with the wrong shape reads as absent:\nreporting status must not fail on a record `records validate` would reject.",
      "properties": {
        "clarificationCount": {
          "description": "Unresolved `[NEEDS CLARIFICATION: …]` markers in its prose.",
          "format": "uint",
          "minimum": 0,
          "type": "integer"
        },
        "id": {
          "type": [
            "string",
            "null"
          ]
        },
        "pullRequest": {
          "description": "The `pull-request` the proposal names: a number or a URL.",
          "type": [
            "string",
            "null"
          ]
        },
        "status": {
          "type": [
            "string",
            "null"
          ]
        },
        "supersedes": {
          "items": {
            "type": "string"
          },
          "type": [
            "array",
            "null"
          ]
        },
        "title": {
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "clarificationCount"
      ],
      "type": "object"
    },
    "ProposalSelection": {
      "description": "How the proposal was chosen, when the choice was not certain.",
      "properties": {
        "otherCandidates": {
          "description": "Other unfinished proposals the heuristic passed over.",
          "items": {
            "$ref": "#/$defs/ProposalSummary"
          },
          "type": "array"
        },
        "pullRequestMismatch": {
          "description": "Whether the chosen proposal names a different pull request than the branch's.",
          "type": "boolean"
        }
      },
      "required": [
        "otherCandidates",
        "pullRequestMismatch"
      ],
      "type": "object"
    },
    "ProposalSummary": {
      "properties": {
        "id": {
          "type": [
            "string",
            "null"
          ]
        },
        "title": {
          "type": [
            "string",
            "null"
          ]
        }
      },
      "type": "object"
    },
    "PullRequest": {
      "properties": {
        "baseRefName": {
          "type": "string"
        },
        "isDraft": {
          "type": "boolean"
        },
        "number": {
          "format": "uint64",
          "minimum": 0,
          "type": "integer"
        },
        "state": {
          "$ref": "#/$defs/PullRequestState"
        },
        "title": {
          "type": "string"
        },
        "url": {
          "type": "string"
        }
      },
      "required": [
        "number",
        "state",
        "baseRefName",
        "isDraft",
        "url",
        "title"
      ],
      "type": "object"
    },
    "PullRequestLookup": {
      "description": "The branch's pull request, as far as `gh` could tell.",
      "oneOf": [
        {
          "$ref": "#/$defs/PullRequest",
          "description": "The branch has exactly this pull request, open or not.",
          "properties": {
            "status": {
              "const": "found",
              "type": "string"
            }
          },
          "required": [
            "status"
          ],
          "type": "object"
        },
        {
          "description": "`gh` positively found no pull request for the branch.",
          "properties": {
            "status": {
              "const": "none",
              "type": "string"
            }
          },
          "required": [
            "status"
          ],
          "type": "object"
        },
        {
          "description": "`gh` could not tell, so whether a pull request exists is unknown.",
          "properties": {
            "reason": {
              "type": "string"
            },
            "status": {
              "const": "unknown",
              "type": "string"
            }
          },
          "required": [
            "status",
            "reason"
          ],
          "type": "object"
        }
      ]
    },
    "PullRequestState": {
      "enum": [
        "OPEN",
        "CLOSED",
        "MERGED"
      ],
      "type": "string"
    },
    "ReadyItem": {
      "properties": {
        "id": {
          "type": "string"
        },
        "priority": {
          "format": "int64",
          "type": "integer"
        },
        "title": {
          "type": "string"
        }
      },
      "required": [
        "id",
        "title",
        "priority"
      ],
      "type": "object"
    },
    "SyncOutcome": {
      "oneOf": [
        {
          "const": "synced",
          "description": "This replica pulled every change on the remote and published its own.",
          "type": "string"
        },
        {
          "const": "skipped",
          "description": "`--no-sync` was passed; nothing was pulled or pushed.",
          "type": "string"
        },
        {
          "const": "failed",
          "description": "The sync did not finish, or was refused before it started.",
          "type": "string"
        },
        {
          "const": "conflict",
          "description": "Beads halted on a conflict it cannot settle; nothing was pushed.",
          "type": "string"
        },
        {
          "const": "blocked",
          "description": "A dirty working set stops Beads from publishing; nothing was pushed.",
          "type": "string"
        }
      ]
    },
    "SyncResult": {
      "description": "How one sync ended.",
      "properties": {
        "detail": {
          "description": "What happened, in words; for anything but `synced`, why.",
          "type": "string"
        },
        "outcome": {
          "$ref": "#/$defs/SyncOutcome"
        }
      },
      "required": [
        "outcome",
        "detail"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "The status report.",
  "properties": {
    "actor": {
      "$ref": "#/$defs/Actor"
    },
    "beads": {
      "$ref": "#/$defs/BeadsReport"
    },
    "facts": {
      "$ref": "#/$defs/Facts"
    },
    "next": {
      "type": "string"
    },
    "notices": {
      "items": {
        "$ref": "#/$defs/Notice"
      },
      "type": "array"
    },
    "phase": {
      "type": "string"
    },
    "reason": {
      "description": "Why the pull request is unknown, when it is.",
      "type": [
        "string",
        "null"
      ]
    },
    "situation": {
      "type": "string"
    },
    "skill": {
      "description": "The skill that covers the next action, by name.",
      "type": [
        "string",
        "null"
      ]
    },
    "title": {
      "type": "string"
    }
  },
  "required": [
    "phase",
    "title",
    "situation",
    "next",
    "actor",
    "notices",
    "facts",
    "beads"
  ],
  "title": "WorkflowStatus",
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

### `project`

Set up and check a project's use of Workbench.

#### `workbench project validate`

Check the project's Workbench setup: pin, link, harness declarations, names, vendored files.

Checks how the project uses Workbench and reports each problem as a violation with a `path` relative to the project root and a `message`: `mise.toml` pinning no `github:markmals/workbench` version (fix: `mise use github:markmals/workbench@<version>`); a `.workbench` link that is missing, dangling, or holds a different version than the pin (the link's version is the `version` in its `.claude-plugin/plugin.json`; fix: `mise install`); a missing or unreadable harness declaration (the `workbench` marketplace and the `workbench@workbench` plugin in `.claude/settings.json`, the `./.workbench` extension in `.omp/config.yml`); a project skill `.agents/skills/<name>/` or rule `.agents/rules/<name>.md` that reuses the name of a skill or rule this release ships; and vendored Workbench files the package now supplies: templates in `.agents/templates/` identical to the package's (a different one is an override and passes), the Oh My Pi extensions `check-before-stop`, `format-after-edit`, and `prime-beads`, `.omp/tools/dynamic-workflows/`, the TypeScript tools this CLI replaces and their tests, `.changeset/config.json`, `docs/.vitepress/`, and `fnox.toml`. Templates are compared with `.workbench/templates/`, or with the copies built into workbench from the same release when the link is missing. Workbench's own repository, whose `.workbench` links to its working tree's `plugin/`, has no pin and skips the pin and link check. Any violation fails with CHECK_FAILED, and the result still goes to stdout. Never changes anything.

- **Effect:** reads only
- **Requires:** `git`
- **Dry run:** no
- **Idempotent:** yes
- **Errors:** `CHECK_FAILED`, `INVALID_ARGUMENT`, `PRECONDITION_FAILED`, `DEPENDENCY_MISSING`, `INTERNAL`

Examples:

```sh
# Check the project's Workbench setup
workbench project validate
```

Result schema:

```json
{
  "$defs": {
    "Violation": {
      "description": "One problem in one file.",
      "properties": {
        "message": {
          "type": "string"
        },
        "path": {
          "description": "The file or directory, relative to the project root.",
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
  "description": "The project's Workbench setup problems.",
  "properties": {
    "count": {
      "description": "How many violations there are; zero means the setup is valid.",
      "format": "uint",
      "minimum": 0,
      "type": "integer"
    },
    "violations": {
      "description": "Every violation, in the order the checks found them.",
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
  "title": "Validation",
  "type": "object"
}
```

#### `workbench project init`

Add Workbench to this Git repository without overwriting anything.

Adds what Workbench owns to the Git repository, creating each missing item and never overwriting or deleting a file. Runs only where `mise.toml` pins `github:markmals/workbench` (otherwise PRECONDITION_FAILED; `mise use github:markmals/workbench@<version>` writes the pin). To existing structured files it adds only missing keys and entries: `mise.toml` gets the `postinstall` hook that links `.workbench`, pins for hk, gh, and Beads, and the `status`, `validate`, `beads:sync`, `beads:claim`, and `beads:close` tasks; `.gitignore` gets `.workbench`; `.claude/settings.json` gets the `workbench` marketplace and the `workbench@workbench` plugin; `.omp/config.yml` gets the `./.workbench` extension. It links `.claude/skills` and `.claude/rules` to `../.agents/skills` and `../.agents/rules` when those directories exist. It creates, when absent: `hk.pkl` (the Scoped Commits check on `commit-msg` and the Beads hook steps and syncs), `.github/workflows/workbench.yml` (commit and change-file checks on every pull request but Knope's release pull request), an `AGENTS.md` stub with the `.workbench` line, `VISION.md` from the template (`.agents/templates/VISION.md` when the project overrides it) with today's date in place of `<YYYY-MM-DD>`, so record validation accepts it, the `proposals/`, `policies/`, `decisions/`, and `guides/` READMEs, `.agents/commit-scopes`, and `docs/hugo.toml`. `--site github-pages` or `--site cloudflare` also adds the `hugo` and `github:CloudCannon/pagefind` pins, the `docs:build` and `docs:dev` tasks, `docs/.gitignore` for the build output in `docs/public`, and that host's publishing and preview workflows, `.github/workflows/docs.yml` and `preview.yml`, which pass the base URL through `HUGO_BASEURL`; Cloudflare also gets `docs/wrangler.jsonc`, the Worker that serves the site. The receipt lists what it created, what it added to with the entries added, what it left unchanged, what needs a person's attention (such as an `AGENTS.md` without the `.workbench` line), every Workbench file the project still vendors (templates marked `identical`, removable, or `different`, an override), and the project-owned pieces Workbench relies on but does not create: the `setup`, `check`, and `fmt` tasks and a release flow that consumes change files with Knope. `--dry-run` returns the same receipt with status `dry-run` and writes nothing. Exits 0 unless a file could not be written, which fails with PRECONDITION_FAILED while the receipt still reports what was written. Afterwards, `mise install` links `.workbench`, and `workbench project validate` and `workbench context show` read the setup back.

- **Effect:** modifies project files: Workbench's mise, Git, and harness declarations, hooks, CI workflow, and record scaffolding
- **Requires:** `git`
- **Dry run:** yes
- **Idempotent:** yes
- **Errors:** `INVALID_ARGUMENT`, `PRECONDITION_FAILED`, `DEPENDENCY_MISSING`, `INTERNAL`

| Argument | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `--site <choice>` | `github-pages`, `cloudflare` | no | — | Also set up the documentation site and its publishing and preview workflows on this host |
| `--dry-run` | boolean | no | — | Validate input, resolve the target, and describe the changes without making them |

Examples:

```sh
# Preview what adopting Workbench would create and add
workbench project init --dry-run
# Preview adoption with a documentation site on GitHub Pages
workbench project init --site github-pages --dry-run
```

Result schema:

```json
{
  "$defs": {
    "Addition": {
      "description": "An existing file and the entries added to it.",
      "properties": {
        "entries": {
          "description": "The keys or lines added, such as `tools.hk`, `tasks.\"beads:sync\"`, or `.workbench`.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "path": {
          "type": "string"
        }
      },
      "required": [
        "path",
        "entries"
      ],
      "type": "object"
    },
    "MissingPiece": {
      "description": "A project-owned piece Workbench relies on but does not create.",
      "properties": {
        "message": {
          "type": "string"
        },
        "piece": {
          "description": "`setup`, `check`, or `fmt` for a mise task; `release` for the Knope release flow.",
          "type": "string"
        }
      },
      "required": [
        "piece",
        "message"
      ],
      "type": "object"
    },
    "Notice": {
      "description": "Something about one file a person needs to know or do.",
      "properties": {
        "message": {
          "type": "string"
        },
        "path": {
          "type": "string"
        }
      },
      "required": [
        "path",
        "message"
      ],
      "type": "object"
    },
    "Status": {
      "oneOf": [
        {
          "const": "initialized",
          "description": "It created or added to files.",
          "type": "string"
        },
        {
          "const": "unchanged",
          "description": "Everything was already in place; nothing was written.",
          "type": "string"
        },
        {
          "const": "dry-run",
          "description": "`--dry-run`: nothing was written.",
          "type": "string"
        }
      ]
    },
    "TemplateMatch": {
      "enum": [
        "identical",
        "different"
      ],
      "type": "string"
    },
    "VendoredFile": {
      "description": "One vendored file or directory.",
      "properties": {
        "kind": {
          "$ref": "#/$defs/VendoredKind"
        },
        "message": {
          "description": "What the file is and what to do with it.",
          "type": "string"
        },
        "path": {
          "description": "Relative to the project root; a directory has no trailing slash.",
          "type": "string"
        },
        "template": {
          "anyOf": [
            {
              "$ref": "#/$defs/TemplateMatch"
            },
            {
              "type": "null"
            }
          ],
          "description": "For a template: `identical` to the package's, so removable, or `different`, an override\nthe project keeps. Null for every other kind."
        }
      },
      "required": [
        "path",
        "kind",
        "message"
      ],
      "type": "object"
    },
    "VendoredKind": {
      "oneOf": [
        {
          "enum": [
            "template",
            "changesets-config",
            "vitepress",
            "fnox"
          ],
          "type": "string"
        },
        {
          "const": "skill",
          "description": "A project skill named like a Workbench skill.",
          "type": "string"
        },
        {
          "const": "rule",
          "description": "A project rule named like a Workbench rule.",
          "type": "string"
        },
        {
          "const": "omp-extension",
          "description": "One of the Oh My Pi extensions Workbench no longer uses.",
          "type": "string"
        },
        {
          "const": "workflow-tool",
          "description": "The Oh My Pi workflow tool.",
          "type": "string"
        },
        {
          "const": "tool",
          "description": "A TypeScript tool the `workbench` CLI replaces, or its test.",
          "type": "string"
        }
      ]
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "description": "What `project init` did, or for a dry run, what it would do.",
  "properties": {
    "addedTo": {
      "description": "Existing files given missing entries.",
      "items": {
        "$ref": "#/$defs/Addition"
      },
      "type": "array"
    },
    "attention": {
      "description": "Existing content left as it was that a person needs to complete by hand.",
      "items": {
        "$ref": "#/$defs/Notice"
      },
      "type": "array"
    },
    "cannotPredict": {
      "description": "What a dry run cannot predict; empty for a real run.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "created": {
      "description": "Files and links created, relative to the project root.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "failed": {
      "description": "Files that could not be written, with the reason. Empty unless the command failed.",
      "items": {
        "$ref": "#/$defs/Notice"
      },
      "type": "array"
    },
    "missing": {
      "description": "Project-owned pieces Workbench relies on but does not create.",
      "items": {
        "$ref": "#/$defs/MissingPiece"
      },
      "type": "array"
    },
    "project": {
      "description": "The project root the paths are relative to.",
      "type": "string"
    },
    "site": {
      "description": "The documentation site host requested with `--site`, or null.",
      "type": [
        "string",
        "null"
      ]
    },
    "status": {
      "$ref": "#/$defs/Status"
    },
    "unchanged": {
      "description": "Items already in place, left as they were.",
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "vendored": {
      "description": "Workbench files the project still vendors.",
      "items": {
        "$ref": "#/$defs/VendoredFile"
      },
      "type": "array"
    }
  },
  "required": [
    "status",
    "project",
    "created",
    "addedTo",
    "unchanged",
    "attention",
    "failed",
    "vendored",
    "missing",
    "cannotPredict"
  ],
  "title": "Receipt",
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

#### `workbench commits scopes`

List the scopes a commit subject may use, and where each comes from.

Lists every scope `workbench commits check` accepts, in the order it reads them: the built-in process scopes (`source: built-in`); the scopes listed one per line in the optional `.agents/commit-scopes` (`source: scopes-file`); and `<NNNN>-<slug>` for every `proposals/<NNNN>-<slug>.md` (`source: proposal`, with the proposal's `path`). A scope the scopes file repeats is listed once, with its first source. Run it before writing a commit subject; a scope a project needs that no list has goes into `.agents/commit-scopes` in the same commit. Reads only. An unreadable `.agents/commit-scopes` or `proposals/` is PRECONDITION_FAILED.

- **Effect:** reads only
- **Requires:** `git`
- **Dry run:** no
- **Idempotent:** yes
- **Errors:** `PRECONDITION_FAILED`, `DEPENDENCY_MISSING`, `INTERNAL`

Examples:

```sh
# List the scopes this project's commit subjects may use
workbench commits scopes
```

Result schema:

```json
{
  "$defs": {
    "Scope": {
      "description": "One accepted scope.",
      "properties": {
        "path": {
          "description": "For a proposal scope, the proposal file it names.",
          "type": [
            "string",
            "null"
          ]
        },
        "scope": {
          "description": "The scope, as written before the colon in a subject.",
          "type": "string"
        },
        "source": {
          "$ref": "#/$defs/Source"
        }
      },
      "required": [
        "scope",
        "source"
      ],
      "type": "object"
    },
    "Source": {
      "description": "Where an accepted scope comes from.",
      "oneOf": [
        {
          "const": "built-in",
          "description": "One of Workbench's process scopes, accepted in every project.",
          "type": "string"
        },
        {
          "const": "scopes-file",
          "description": "Listed in the project's `.agents/commit-scopes`.",
          "type": "string"
        },
        {
          "const": "proposal",
          "description": "The `<NNNN>-<slug>` of a proposal in `proposals/`.",
          "type": "string"
        }
      ]
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "properties": {
    "items": {
      "description": "Every accepted scope, built-in first, then the scopes file's, then the proposals'.",
      "items": {
        "$ref": "#/$defs/Scope"
      },
      "type": "array"
    }
  },
  "required": [
    "items"
  ],
  "title": "CommitScopes",
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
| `--body <text>` | text | no | — | Detail below the heading: the text, `@<file>` to read a file, or `-` to read stdin |
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

Passes when the range, the commits reachable from head and not from base, either adds a valid change file, a Markdown file directly under `.changeset/`, or has a commit carrying the Git trailer `Release-Note: none` (the key in any case, the value exactly `none`, parsed by Git the way `git interpret-trailers --parse` parses it). Added files are the ones head adds since its merge base with base, read as committed at head. A valid change file has valid frontmatter, then a summary: the first non-blank line after the frontmatter is a heading with text, such as `# Add the export queue`, which Knope writes into the changelog. Valid frontmatter is a `---` line, one `<package>: <type>` line per affected package, and a closing `---` line; when the project has a knope.toml, every package must be one it declares (`default` for a single `[package]`) and every type `major`, `minor`, `patch`, or one that package's `extra_changelog_sections` add. The result lists each added change file with its problems and the commits carrying the trailer. Otherwise it fails with CHECK_FAILED naming both ways to satisfy it. When the check passes but another added file is invalid, stderr warns about it, because Knope fails on it. An unknown revision is NOT_FOUND; base and head without shared history is INVALID_ARGUMENT; an unusable knope.toml is PRECONDITION_FAILED. CI runs it on every pull request except Knope's release pull request.

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

### `beads`

This machine's Beads replica: sync, claim, and close.

#### `workbench beads sync`

Sync this machine's Beads replica with the project's remote.

Pulls every change other machines published to the project's Beads remote (`refs/dolt/data` on the checkout's `origin`) and publishes this replica's, with `bd sync`; when the remote has no Beads history yet, the first sync publishes it with `bd dolt push`. Refuses, changing nothing, when the replica's Beads remote is not the checkout's `origin` or there is no `origin`. Limited to `--timeout` seconds (default 20). Exits 0 only when the receipt's `status` is `synced`. Otherwise the error's `details.outcome` is `failed`, `conflict`, or `blocked`, and its code says why: UNAVAILABLE (remote or network unreachable, push retries exhausted), TIMEOUT, UNAUTHENTICATED (credentials rejected), PRECONDITION_FAILED (a foreign remote or no `origin`, no Beads workspace, or a dirty working set that blocks publishing), DEPENDENCY_MISSING (no `bd`), or CONFLICT (a halted merge; `details.tables` and `details.differing` name what conflicts). `--dry-run` reports the remote and the changes this replica has not published, without syncing.

- **Effect:** modifies this machine's Beads replica and the project's Beads remote
- **Requires:** `git`, `bd`, `network`
- **Dry run:** yes
- **Idempotent:** yes
- **Errors:** `INVALID_ARGUMENT`, `PRECONDITION_FAILED`, `DEPENDENCY_MISSING`, `UNAUTHENTICATED`, `CONFLICT`, `TIMEOUT`, `UNAVAILABLE`, `INTERNAL`

| Argument | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `--dry-run` | boolean | no | — | Validate input, resolve the target, and describe the changes without making them |
| `--timeout <integer>` | integer | no | `20` | Seconds each sync with the Beads remote may take before it fails |

Examples:

```sh
# Publish what this machine changed and pull what the others did
workbench beads sync
# See the remote and the unpublished changes without syncing
workbench beads sync --dry-run
```

Result schema:

```json
{
  "$defs": {
    "DifferingItem": {
      "description": "An item that differs between two states of the database.",
      "properties": {
        "change": {
          "description": "How it differs, as Beads names it: `added`, `removed`, or `modified`.",
          "type": [
            "string",
            "null"
          ]
        },
        "id": {
          "type": "string"
        },
        "title": {
          "description": "Its title on whichever side has it.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "id"
      ],
      "type": "object"
    },
    "SyncOutcome": {
      "oneOf": [
        {
          "const": "synced",
          "description": "This replica pulled every change on the remote and published its own.",
          "type": "string"
        },
        {
          "const": "skipped",
          "description": "`--no-sync` was passed; nothing was pulled or pushed.",
          "type": "string"
        },
        {
          "const": "failed",
          "description": "The sync did not finish, or was refused before it started.",
          "type": "string"
        },
        {
          "const": "conflict",
          "description": "Beads halted on a conflict it cannot settle; nothing was pushed.",
          "type": "string"
        },
        {
          "const": "blocked",
          "description": "A dirty working set stops Beads from publishing; nothing was pushed.",
          "type": "string"
        }
      ]
    },
    "SyncPreview": {
      "properties": {
        "cannotPredict": {
          "description": "What the dry run cannot know until the sync runs.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "project": {
          "type": "string"
        },
        "remote": {
          "type": [
            "string",
            "null"
          ]
        },
        "status": {
          "description": "`dry-run`.",
          "type": "string"
        },
        "unpublished": {
          "$ref": "#/$defs/UnpublishedChanges"
        }
      },
      "required": [
        "status",
        "project",
        "unpublished",
        "cannotPredict"
      ],
      "type": "object"
    },
    "Synced": {
      "properties": {
        "detail": {
          "type": "string"
        },
        "project": {
          "description": "The project root.",
          "type": "string"
        },
        "remote": {
          "description": "The Beads remote, without credentials; `null` when there is none.",
          "type": [
            "string",
            "null"
          ]
        },
        "status": {
          "$ref": "#/$defs/SyncOutcome",
          "description": "`synced`."
        }
      },
      "required": [
        "status",
        "project",
        "detail"
      ],
      "type": "object"
    },
    "UnpublishedChanges": {
      "description": "The changes this replica has not published, as far as it knows.",
      "properties": {
        "determined": {
          "description": "Whether they could be determined. Beads compares this replica with the remote as last\nfetched (`bd diff origin/main main`), by item rather than by commit.",
          "type": "boolean"
        },
        "items": {
          "description": "The items that differ from the remote as last fetched.",
          "items": {
            "$ref": "#/$defs/DifferingItem"
          },
          "type": "array"
        },
        "reason": {
          "description": "Why they could not be determined.",
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "determined",
        "items"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "anyOf": [
    {
      "$ref": "#/$defs/Synced"
    },
    {
      "$ref": "#/$defs/SyncPreview"
    }
  ],
  "description": "A completed sync, or a dry run's description of one.",
  "title": "SyncReceipt"
}
```

#### `workbench beads claim`

Claim a Beads item as this machine's claimant, so other machines see the claim.

Syncs, claims the item as this machine's claimant (`BEADS_ACTOR`, or `<login user>@<short host name>` when it is unset), syncs again, and reads the item back to confirm who holds it. The receipt's `status` is `claimed` (`published: true`) or, when the second sync failed, `claimed-unpublished` (`published: false`, with the reason on stderr): both exit 0 and the item is yours. CONFLICT (exit 4) means another claimant already held the item (`details.outcome: held`) or a concurrent claim from another machine won (`lost`); `details.holder` names the holder. Do not start the work then. An unknown id is NOT_FOUND; another refusal from Beads, or a replica whose Beads remote is not the checkout's `origin`, is PRECONDITION_FAILED and changes nothing. Each sync is limited to `--timeout` seconds (default 20). `--dry-run` reports the item, its current claimant, and the claimant it would claim as.

- **Effect:** modifies the item's assignee and status, in this replica and on the Beads remote
- **Requires:** `git`, `bd`, `network`
- **Dry run:** yes
- **Idempotent:** yes
- **Errors:** `INVALID_ARGUMENT`, `NOT_FOUND`, `PRECONDITION_FAILED`, `DEPENDENCY_MISSING`, `CONFLICT`, `INTERNAL`

| Argument | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `<id>` | string | yes | — | The Beads item's id, such as wb-1 |
| `--dry-run` | boolean | no | — | Validate input, resolve the target, and describe the changes without making them |
| `--timeout <integer>` | integer | no | `20` | Seconds each sync with the Beads remote may take before it fails |

Examples:

```sh
# Claim wb-1 before starting it
workbench beads claim wb-1
# See who holds wb-1 and who the claim would be made as
workbench beads claim wb-1 --dry-run
```

Result schema:

```json
{
  "$defs": {
    "ItemChange": {
      "description": "A change Beads accepted.",
      "properties": {
        "claimant": {
          "description": "Who the change was made as.",
          "type": "string"
        },
        "detail": {
          "type": "string"
        },
        "id": {
          "description": "The item.",
          "type": "string"
        },
        "project": {
          "description": "The project root.",
          "type": "string"
        },
        "published": {
          "description": "Whether other machines can see the change.",
          "type": "boolean"
        },
        "remote": {
          "description": "The Beads remote, without credentials; `null` when there is none.",
          "type": [
            "string",
            "null"
          ]
        },
        "status": {
          "description": "`claimed` or `closed` when other machines can see the change; `claimed-unpublished` or\n`closed-unpublished` when only this machine has it because the sync failed.",
          "type": "string"
        },
        "sync": {
          "$ref": "#/$defs/SyncResult",
          "description": "The sync that published the change, or failed to."
        }
      },
      "required": [
        "status",
        "project",
        "id",
        "claimant",
        "published",
        "sync",
        "detail"
      ],
      "type": "object"
    },
    "ItemPreview": {
      "description": "What a claim or close would act on.",
      "properties": {
        "cannotPredict": {
          "description": "What the dry run cannot know until the change runs.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "claimant": {
          "description": "The claimant the change would be made as.",
          "type": "string"
        },
        "currentClaimant": {
          "description": "Who holds the item in this replica now, if anyone.",
          "type": [
            "string",
            "null"
          ]
        },
        "id": {
          "type": "string"
        },
        "itemStatus": {
          "description": "The item's status in this replica: `open`, `in_progress`, `closed`, and so on.",
          "type": [
            "string",
            "null"
          ]
        },
        "project": {
          "type": "string"
        },
        "remote": {
          "type": [
            "string",
            "null"
          ]
        },
        "status": {
          "description": "`dry-run`.",
          "type": "string"
        },
        "title": {
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "status",
        "project",
        "id",
        "claimant",
        "cannotPredict"
      ],
      "type": "object"
    },
    "SyncOutcome": {
      "oneOf": [
        {
          "const": "synced",
          "description": "This replica pulled every change on the remote and published its own.",
          "type": "string"
        },
        {
          "const": "skipped",
          "description": "`--no-sync` was passed; nothing was pulled or pushed.",
          "type": "string"
        },
        {
          "const": "failed",
          "description": "The sync did not finish, or was refused before it started.",
          "type": "string"
        },
        {
          "const": "conflict",
          "description": "Beads halted on a conflict it cannot settle; nothing was pushed.",
          "type": "string"
        },
        {
          "const": "blocked",
          "description": "A dirty working set stops Beads from publishing; nothing was pushed.",
          "type": "string"
        }
      ]
    },
    "SyncResult": {
      "description": "How one sync ended.",
      "properties": {
        "detail": {
          "description": "What happened, in words; for anything but `synced`, why.",
          "type": "string"
        },
        "outcome": {
          "$ref": "#/$defs/SyncOutcome"
        }
      },
      "required": [
        "outcome",
        "detail"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "anyOf": [
    {
      "$ref": "#/$defs/ItemChange"
    },
    {
      "$ref": "#/$defs/ItemPreview"
    }
  ],
  "description": "A claim or close, made or previewed.",
  "title": "ItemReceipt"
}
```

#### `workbench beads close`

Close a Beads item as this machine's claimant, then sync so other machines see it.

Closes the item with the reason as this machine's claimant (`BEADS_ACTOR`, or `<login user>@<short host name>` when it is unset) — a plain `bd close` runs as the Git user name and is refused for an item the claimant holds — then syncs. The receipt's `status` is `closed` (`published: true`) or, when the sync failed, `closed-unpublished` (`published: false`, with the reason on stderr); both exit 0. When Beads refuses because another claimant holds the item the error is CONFLICT, with `details.holder`; an unknown id is NOT_FOUND; another refusal, or a replica whose Beads remote is not the checkout's `origin`, is PRECONDITION_FAILED. A refused close syncs nothing. The sync is limited to `--timeout` seconds (default 20). `--dry-run` reports the item, its current claimant, and the claimant it would close as.

- **Effect:** modifies the item's status, in this replica and on the Beads remote
- **Requires:** `git`, `bd`, `network`
- **Dry run:** yes
- **Idempotent:** yes
- **Errors:** `INVALID_ARGUMENT`, `NOT_FOUND`, `PRECONDITION_FAILED`, `DEPENDENCY_MISSING`, `CONFLICT`, `INTERNAL`

| Argument | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `<id>` | string | yes | — | The Beads item's id, such as wb-1 |
| `--reason <string>` | string | yes | — | Why the item is closed, such as the pull request that resolved it |
| `--dry-run` | boolean | no | — | Validate input, resolve the target, and describe the changes without making them |
| `--timeout <integer>` | integer | no | `20` | Seconds each sync with the Beads remote may take before it fails |

Examples:

```sh
# Close wb-1 once its fix is in a pull request
workbench beads close wb-1 --reason 'Fixed in PR #42'
# See who holds wb-1 and who the close would be made as
workbench beads close wb-1 --reason Done --dry-run
```

Result schema:

```json
{
  "$defs": {
    "ItemChange": {
      "description": "A change Beads accepted.",
      "properties": {
        "claimant": {
          "description": "Who the change was made as.",
          "type": "string"
        },
        "detail": {
          "type": "string"
        },
        "id": {
          "description": "The item.",
          "type": "string"
        },
        "project": {
          "description": "The project root.",
          "type": "string"
        },
        "published": {
          "description": "Whether other machines can see the change.",
          "type": "boolean"
        },
        "remote": {
          "description": "The Beads remote, without credentials; `null` when there is none.",
          "type": [
            "string",
            "null"
          ]
        },
        "status": {
          "description": "`claimed` or `closed` when other machines can see the change; `claimed-unpublished` or\n`closed-unpublished` when only this machine has it because the sync failed.",
          "type": "string"
        },
        "sync": {
          "$ref": "#/$defs/SyncResult",
          "description": "The sync that published the change, or failed to."
        }
      },
      "required": [
        "status",
        "project",
        "id",
        "claimant",
        "published",
        "sync",
        "detail"
      ],
      "type": "object"
    },
    "ItemPreview": {
      "description": "What a claim or close would act on.",
      "properties": {
        "cannotPredict": {
          "description": "What the dry run cannot know until the change runs.",
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "claimant": {
          "description": "The claimant the change would be made as.",
          "type": "string"
        },
        "currentClaimant": {
          "description": "Who holds the item in this replica now, if anyone.",
          "type": [
            "string",
            "null"
          ]
        },
        "id": {
          "type": "string"
        },
        "itemStatus": {
          "description": "The item's status in this replica: `open`, `in_progress`, `closed`, and so on.",
          "type": [
            "string",
            "null"
          ]
        },
        "project": {
          "type": "string"
        },
        "remote": {
          "type": [
            "string",
            "null"
          ]
        },
        "status": {
          "description": "`dry-run`.",
          "type": "string"
        },
        "title": {
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "status",
        "project",
        "id",
        "claimant",
        "cannotPredict"
      ],
      "type": "object"
    },
    "SyncOutcome": {
      "oneOf": [
        {
          "const": "synced",
          "description": "This replica pulled every change on the remote and published its own.",
          "type": "string"
        },
        {
          "const": "skipped",
          "description": "`--no-sync` was passed; nothing was pulled or pushed.",
          "type": "string"
        },
        {
          "const": "failed",
          "description": "The sync did not finish, or was refused before it started.",
          "type": "string"
        },
        {
          "const": "conflict",
          "description": "Beads halted on a conflict it cannot settle; nothing was pushed.",
          "type": "string"
        },
        {
          "const": "blocked",
          "description": "A dirty working set stops Beads from publishing; nothing was pushed.",
          "type": "string"
        }
      ]
    },
    "SyncResult": {
      "description": "How one sync ended.",
      "properties": {
        "detail": {
          "description": "What happened, in words; for anything but `synced`, why.",
          "type": "string"
        },
        "outcome": {
          "$ref": "#/$defs/SyncOutcome"
        }
      },
      "required": [
        "outcome",
        "detail"
      ],
      "type": "object"
    }
  },
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "anyOf": [
    {
      "$ref": "#/$defs/ItemChange"
    },
    {
      "$ref": "#/$defs/ItemPreview"
    }
  ],
  "description": "A claim or close, made or previewed.",
  "title": "ItemReceipt"
}
```
