---
status: accepted
---

# 0004: Group CLI commands by Prove2me API endpoint

## Context and Problem Statement

The [Prove2me CLI](../prd/prove2me-cli.md) is primarily for agents, but people also use it directly. Both should be able to connect a command to the existing API documentation without constructing HTTP requests. The CLI must also distinguish a failed command from a proof that Prove2me verified and rejected.

## Considered Options

1. **Group by endpoint and use operation verbs.** `verify status` points to `/verify` and explains what the call does.
2. **Group by endpoint and use HTTP methods.** `verify get` matches the HTTP method but does not say what the caller gets.
3. **Use flat task commands.** `status` is short, but the endpoint is not visible in the name.
4. **Expose a generic API command.** `api GET` accepts the API path directly, but callers still have to construct paths, queries, and request bodies.

## Decision Outcome

Use endpoint groups with operation verbs. Wrap the documented Prove2me API; do not change it or expose arbitrary paths. The table shows the required inputs for the first-version commands. Paths are relative to `/api/v1`.

| CLI command | Prove2me API | Input mapping |
| --- | --- | --- |
| `p2me theorems search <query>` | `GET /theorems` | Send the keyword query as `q`. |
| `p2me theorems get <theorem-id>` | `GET /theorems/{theorem_id}` | Use the theorem ID in the path. |
| `p2me verify submit <theorem-id> <file>` | `POST /verify` | Upload `theorem_id` and `file` as multipart form fields. Also support `proof_type` and `explanation` as described in [ADR 0002](0002-use-an-explicit-verification-workflow.md). |
| `p2me verify status <submission-id>` | `GET /verify` | Send the ID as `submission_id` and fetch the status once. |

The [coverage matrix](../prove2me-api-coverage.md) lists the rest of the endpoint groups and their request parameters. Commands accept path IDs as positional arguments, named query flags, and a JSON object from a file or stdin when the API expects a JSON body. There is no arbitrary-method or arbitrary-path command.

Print successful API responses that return data as indented JSON, following [ADR 0001](0001-default-to-readable-json.md). Do not rename or wrap fields. The documented upload response calls the ID `submission_id`; the status response calls it `id`. Preserve both names as returned by the API. An HTTP `204 No Content` response succeeds without output.

Use process exit code `0` when the CLI completes the requested operation and returns its result. A successful status lookup exits `0` even if the platform reports `PENDING`, `WA`, or `ERROR`: those values describe verification. If the CLI cannot get or return a result, such as on a network or HTTP error, exit nonzero and write the diagnostic to standard error. Do not put the process exit code in the JSON response.

### Consequences

Agents can reuse field names from the API documentation and use command help to find the supported operations. The CLI does not shield callers from changes to response fields in the upstream API. A command that waits for verification is still a separate proposal; the status command fetches it once.

### Confirmation

Run the built executable against a local test server. Verify the HTTP method, path, and inputs for each command. Check that the indented output parses to the same JSON value as the API response, including each response's original ID field. Verify that a `WA` status response exits `0` and that a failed lookup exits nonzero without emitting a verdict.

## More Information

The mapping follows the bundled [theorem discovery](../../.agents/skills/prove2me/references/discover.md) and [verification](../../.agents/skills/prove2me/references/prove.md) references. Endpoint mappings are tested against a local server; they have not all been tested against the live API.
