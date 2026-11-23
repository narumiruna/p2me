---
status: proposed
---

# 0002: Separate proof uploads from waiting for verification

## Context and Problem Statement

The [Prove2me verification API](../../.agents/skills/prove2me/references/prove.md) accepts a Lean file for a proof or disproof, with an optional written explanation. It can return a submission ID while verification is still pending. The [first version](../prd/prove2me-cli.md) supports those submission inputs. It needs to let people and agents recover the result without uploading the same file again.

## Considered Options

1. **Always wait after an upload.** One command can yield a verdict, but callers must wait even when they only need a receipt.
2. **Return a receipt and provide only one-shot status queries.** Callers can resume later but must write their own polling loop if they need a verdict.
3. **Return a receipt and offer separate, optional waiting.** Callers can keep the submission ID before waiting. The CLI must manage a deadline and handle status lookup errors.

## Decision Outcome

Use the same upload flow for proofs and disproofs. Send the caller's Lean file for the specified theorem, set the requested proof type, and include the explanation if supplied. Return the submission ID and status without waiting. Do not retry an upload automatically, including when the connection fails before the CLI receives a receipt.

Retain one-shot status lookup by submission ID. Propose an optional wait operation that checks the same submission repeatedly and stops at a final platform status or a finite deadline. A deadline or lookup error is not a proof verdict and must not trigger another upload. The caller can resume with the same ID. The wait syntax, polling interval, and timeout are not set here.

Report the status Prove2me returns. In particular, `SKETCH_ACCEPTED` can be a final submission status without the target theorem being fully proved. Waiting for verification does not wait for the sketch's open dependencies to be proved.

### Consequences

A proof, a disproof, and a sketch that imports existing platform lemmas use the same upload and status flow. The CLI does not create or manage the sketch's child theorems as part of verification.

### Confirmation

Use a test server to check proof and disproof uploads, including an explanation. Verify that a missing upload response does not cause a second upload. Query a stored ID after the upload process exits. For the proposed wait operation, test a pending status that becomes final, and a pending status that outlasts the deadline. Neither path should send another upload.
