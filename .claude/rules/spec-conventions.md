# Shared Spec Rules

> **This file is included by every platform's `CLAUDE.md` via `@.claude/rules/spec-conventions.md`.** Keep it short — it loads on every session in every platform directory.

## The compact

- **Specs in `specs/` and `features/<n>/` are the source of truth.** Implementations on every platform must satisfy them.
- **Reverse pointers are mandatory.** Every class, function, or module that realizes a spec carries `// SPEC: <id>`. Tests are tagged with the spec IDs they verify.
- **Use `// SPEC: <id> (deviates: <reason>)` when a platform must differ.** Use `// SPEC: manual` for genuinely platform-specific code with no cross-platform analog.
- **The spec defines what; the test proves it; the implementation satisfies it.** None is the source of truth alone.
- **Web is the reference implementation.** When implementing a spec on iOS or Android, read the web realization for context — but the spec is authoritative.

## Before writing implementation code

1. Read the spec file. Confirm the ID, depends-on chain, and behavior.
2. Read the web reference implementation if one exists.
3. Read the platform's existing patterns for similar specs (look for other `// SPEC:` annotations in the same area).
4. Write the failing tests first, tagged with the spec ID and scenario sub-IDs.
5. Implement the minimum to pass the tests.
6. Verify with the platform's `/sdd-verify` command.

## Before changing a spec

1. Search for the ID in the codebase: `rg 'SPEC: <id>'`.
2. List the affected platforms and tests.
3. Update the spec.
4. Use `/sdd-apply <id> <platform>` for each affected platform — propose changes, do not auto-merge.

## Before changing implementation that has a spec

1. Decide: is this a bug fix that the spec already requires, or a behavior change?
2. If behavior change: update the spec first, then run `/sdd-apply`.
3. If bug fix: just fix it, run `/sdd-verify`, and check no other platforms have the same bug.

## Where to read more

- `specs/CONVENTIONS.md` — full conventions, kind taxonomy, frontmatter schema, drift rules.
- `specs/ARCHITECTURE.md` — layering, data flow, deployment.
- `specs/DESIGN_SYSTEM.md` — tokens, components, parity rules.
