# Prove2me CLI (`p2me`)

**Status:** Draft

## Problem

Agents and people working on Lean proofs need to find a Prove2me theorem, read its formal statement, upload a proof or disproof, and check whether the platform accepted it. The repository's [Prove2me skill](../../.agents/skills/prove2me/SKILL.md) documents that workflow with `curl`. Callers must assemble HTTP requests, handle credentials, and track which ID belongs to the theorem and which belongs to the submission. Receiving a submission ID does not tell the caller whether verification passed.

## Goals and non-goals

- **Primary goal:** An agent with an API key and a Lean file can complete the workflow and read structured results without writing HTTP requests or parsing terminal text.
- **Secondary goal:** A person can complete the same workflow from a terminal using indented JSON results and plain-language command help.
- **Non-goals for the first version:** Write proofs, install or run Lean locally, publish new theorems or definitions, or manage missions. Prove2me remains responsible for verifying submissions.

## Requirements

These are proposed requirements for the first version.

| Requirement | Acceptance check |
| --- | --- |
| **Find a theorem.** Accept a keyword query and return matching theorem names and IDs. | Search against a test server with a known match. Check that its name and ID appear and that the ID can be used to inspect the theorem. |
| **Read the target statement.** Given a theorem ID, show the formal Lean statement and its preamble. | Look up a theorem with a nonempty preamble. Check that both the preamble and statement from the server are present. |
| **Submit a proof or disproof.** Upload a caller-selected `.lean` file for an explicit theorem ID and include a written explanation when supplied. Return the submission ID and status immediately after receiving the upload response. Do not wait for a verdict or retry the upload automatically. | Check that the test server receives the theorem ID, proof type, file, and any explanation. Check that `PENDING` does not appear as an accepted result. Drop the upload response and check that the CLI reports an error without sending the file again. |
| **Check the result later.** Given a submission ID, fetch its status once. Distinguish pending, accepted, accepted as a sketch with unproved dependencies, and unsuccessful verification. Distinguish a verdict from a failure to retrieve it. | In a separate invocation, query the same submission ID with fixtures for `PENDING`, `ACCEPTED`, `SKETCH_ACCEPTED`, and a verification failure such as `CE`. Check that each has a distinct result and that an HTTP failure is not presented as a verdict. |
| **Proposed: Wait on request.** Let the caller wait for a final verification status for an existing submission ID, up to a finite deadline. Do not wait on the sketch's dependencies or upload again. | Check that waiting stops on a final status or at the deadline. A deadline must not produce a false verdict. Check that the CLI sends only status requests while waiting. |
| **Use the CLI without exposing credentials.** Read the API key from the process environment, or from `.env` in the current working directory if the variable is unset. Do not accept account passwords, write credentials, print secrets, or send them outside the official Prove2me API. | Use different dummy keys in the environment and `.env` to verify precedence. Test the `.env` fallback and rejected authentication. Check that neither credential appears in output or reaches a redirect outside the API, and that the CLI does not create or modify a credentials file. |
| **Default to indented JSON.** Write one documented JSON result to standard output for each successful data command. Keep command help in plain text and diagnostics out of standard output. | Run commands with output captured. Check that standard output is indented, parses as one JSON result, and contains no diagnostics. Check that help is usable without credentials. |

## Success measures

- An agent can pass the theorem ID and submission ID between calls using the default JSON output. It can distinguish a queued submission, an accepted result, and an accepted sketch without reading terminal text.
- A person can follow the command help to search for a theorem, read its statement, submit a proof or disproof with an explanation, and check its status in a later invocation using only `p2me` for platform operations.
- These outcomes can be demonstrated with the built executable and a local test server, without a live account or a local Lean installation.

## Dependencies and open questions

- The workflow depends on the Prove2me API for [authentication](../../.agents/skills/prove2me/references/setup.md), [theorem discovery](../../.agents/skills/prove2me/references/discover.md), and [proof submission and status](../../.agents/skills/prove2me/references/prove.md). The requirements use the bundled reference documentation; behavior against the live API has not been verified.
- [ADR 0004](../decisions/0004-map-cli-commands-to-api-endpoints.md) defines the core command names, preserves the API response fields, and separates command exit codes from platform verdicts. The [README](../../README.md#submit-a-proof) documents the supported flags and the API key variable. Waiting remains a proposal; its syntax and timeout are open.
- Related ADRs cover [default output](../decisions/0001-default-to-readable-json.md), [submission and verification](../decisions/0002-use-an-explicit-verification-workflow.md), and [credential sources](../decisions/0003-load-api-keys-from-environment-and-dotenv.md).
