---
status: proposed
---

# 0001: Default to indented JSON for command results

## Context and Problem Statement

The [Prove2me CLI](../prd/prove2me-cli.md) is primarily for agents, but people should also be able to use it directly from a terminal. An agent needs a predictable format that does not depend on the terminal. A person needs output that is easy to scan and help that shows how to run the commands.

## Considered Options

1. **Text by default, JSON on request.** Text can be tailored to each command, but every automated call has to request JSON.
2. **Compact JSON by default.** The result is easy to parse and smaller, but a long result is hard to scan in a terminal.
3. **Indented JSON by default.** People and agents get the same structure, with whitespace that makes it easier to read. It uses more output space than compact JSON.

## Decision Outcome

Default to option 3. Every successful command that returns data writes one indented JSON document to standard output, whether run in a terminal or through a pipe. Do not mix diagnostic or progress messages into that stream. Keep command help in plain language with usage examples, and send diagnostics to standard error.

### Consequences

People can read the same results that agents parse. The first version does not need a separate table view. Document command names, JSON fields, and exit codes before callers rely on them; this ADR does not set those details.

### Confirmation

Run a data command with output captured by a test. Parse standard output as one JSON document, verify that it is indented, and check that diagnostics do not appear in it. Review help output for a usage example and check that it is available without credentials.
