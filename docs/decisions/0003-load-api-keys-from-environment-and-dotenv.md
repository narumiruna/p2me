---
status: proposed
---

# 0003: Load API keys from the environment or a local `.env` file

## Context and Problem Statement

The [Prove2me authentication API](../../.agents/skills/prove2me/references/setup.md) exchanges an API key for a short-lived access token. Agents need to inject the key at launch without writing to disk. People also want to reuse settings from a local `.env` file. When both are present, the tool needs a predictable way to choose the key.

## Considered Options

1. **Read only the process environment.** Automation can inject a key, but people must arrange to set the key before each session.
2. **Read the process environment, then fall back to `.env`.** Automation can override local settings, and people can use an existing project file. A key in `.env` is stored as plain text.
3. **Store the key in the system credential store.** This avoids a plain-text file but adds platform-specific setup and does not replace environment injection for unattended use.

## Decision Outcome

Support both sources. Read the named key from the process environment first. Only if it is unset, look for the same key in `.env` in the current working directory. Do not search parent directories. Parse the file as data, not as a shell script. Report a missing or empty key without printing the value. The [README](../../README.md#quick-start) documents `PROVE2ME_API_KEY` and the `.env` syntax.

Exchange the key at the official Prove2me API and use the resulting token for authenticated requests. Do not send either credential outside `https://prove2.me/api/v1`, including through redirects. For ordinary authenticated operations, do not accept account passwords or print credentials. Explicit account commands are separate: `auth login` and related endpoints read JSON from a file or standard input, not password or token flags. Token and key commands return the requested credentials as JSON on standard output, never in diagnostics. The CLI does not write a `.env` file or store credentials on disk.

### Consequences

An agent's explicitly injected key takes precedence over local settings. A user-provided `.env` can contain secrets in plain text even though [this repository ignores the file](../../.gitignore). Document that users should load it only from a trusted working directory and should not commit it.

### Confirmation

Test with dummy keys in the environment only, in `.env` only, and in both sources with different values. Confirm that the environment takes precedence, that a file in a parent directory is not used, and that a missing or empty key yields a diagnostic. Verify that ordinary command output and all diagnostics do not expose credentials, that explicit authentication commands can pass their JSON results through a pipe, and that credentials are never sent to a redirect target.
