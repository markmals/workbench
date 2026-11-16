---
id: proposal.0006
title: Depend on Workbench Instead of Copying It
authors: [markmals, anthropic/claude-opus-5-5]
status: draft
pull-request: https://github.com/markmals/workbench/pull/10
issues: [workbench-0ky]
supersedes: []
---

# Depend on Workbench Instead of Copying It

## Summary

Ship Workbench as a versioned package: a Rust `workbench` CLI plus the plugin content that Claude Code, Oh My Pi, and later Pico load. Projects pin it in `mise.toml` and take updates through that pin, with no Node, TypeScript, or harness extensions.
