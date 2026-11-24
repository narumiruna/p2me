# p2me

`p2me` is a command-line client for [Prove2me](https://prove2.me/). Find theorems, submit Lean proofs or disproofs, and manage missions and collections. Prove2me verifies submissions on its servers, so you do not need to install Lean to use the CLI.

## Install

PyPI and crates.io currently offer only `0.0.0`, a placeholder that prints `Hello, world!`. [Build from a local checkout](#development) to use the CLI now. Once a working version is published, use one of these options in order of preference:

1. **Install with [uv](https://docs.astral.sh/uv/getting-started/installation/) (recommended).** This makes `p2me` available as a command:

   ```sh
   uv tool install p2me
   p2me --help
   ```

2. **Try with `uvx`.** Run `p2me` without installing it as a command. To run the examples below this way, replace `p2me` with `uvx p2me`:

   ```sh
   uvx p2me --help
   ```

3. **Install with Cargo.** If you prefer to build from source with the Rust toolchain, use:

   ```sh
   cargo install --locked p2me
   p2me --help
   ```

## Submit a proof

1. Get an API key from your Prove2me account menu. Set the `PROVE2ME_API_KEY` environment variable, or add this line to a regular `.env` file in the directory where you run `p2me`:

   ```dotenv
   PROVE2ME_API_KEY=your-key
   ```

   In `.env`, use an unquoted value with no spaces or inline comments. `p2me` does not expand variables or search parent directories. The environment variable takes precedence even if it is empty. Do not commit `.env` or use one from an untrusted directory.

2. Search for a theorem. Replace `THEOREM_ID` with an ID from the search results:

   ```sh
   p2me theorems search 'poincare'
   p2me theorems get THEOREM_ID
   ```

3. Submit a regular Lean file that declares `theorem solution`. For a proof, its type must match the target theorem's `formal_statement`. For a disproof, it must prove the negation of the entire statement, including any quantifiers. See the [Prove2me submission guide](.agents/skills/prove2me/references/prove.md) for examples.

   ```sh
   p2me verify submit THEOREM_ID solution.lean
   ```

   The default is `--proof-type prove`. For a disproof, use `--proof-type disprove`. You can add `--explanation` in either mode:

   ```sh
   p2me verify submit THEOREM_ID counterexample.lean --proof-type disprove --explanation 'A counterexample.'
   ```

4. Check the result. Replace `SUBMISSION_ID` with the `submission_id` from the upload response:

   ```sh
   p2me verify status SUBMISSION_ID
   ```

   The upload command returns before verification finishes and does not retry. If the connection fails after the upload starts, check your Prove2me submission history before you resubmit.

### Read the result

Commands that return data write one indented JSON result to standard output. Field names match the API: the upload response uses `submission_id`, and the status response uses `id`.

An exit code of `0` means the status lookup succeeded, not that the submission was accepted. Check the `status` field:

- `ACCEPTED`: Prove2me verified the proof or disproof.
- `PENDING`: Verification is not finished. Run `p2me verify status SUBMISSION_ID` again later.
- `SKETCH_ACCEPTED`: Prove2me accepted a reduction to claims that still need proofs.
- `WA`: The Lean file compiled but did not prove the required statement.

Command failures exit nonzero and write diagnostics to standard error.

## Use other API operations

The [API coverage matrix](docs/prove2me-api-coverage.md) maps all 96 documented operations to CLI commands and lists their request parameters. Run `p2me <group> <command> --help` for command syntax.

1. Pass path IDs as positional arguments and query parameters as flags. Replace underscores in query names with hyphens, for example `--include-deprecated true`.

   ```sh
   p2me theorems search --q 'inner product' --q 'Cauchy Schwarz' --status Open --limit 5
   p2me missions list --visibility public --limit 10
   ```

2. For JSON requests, pass a file containing the API's JSON object, or use `--json-file -` to read it from standard input. The CLI sends field names unchanged.

   ```sh
   printf '%s\n' '{"theorem_id":"THEOREM_ID"}' | p2me saved add --json-file -
   ```

3. Normal commands exchange `PROVE2ME_API_KEY` for a short-lived access token. Public commands such as `p2me health check` do not need a key. To mint a key from an email/password login, pipe the login result to the key command:

   ```sh
   p2me auth login --json-file login.json | p2me auth api-key --session-file -
   ```

   The login input contains `email` and `password`. Protect it, or feed it from standard input. Explicit authentication commands can return tokens or API keys on standard output. Do not send those results to shared logs or commit them.

A successful HTTP `204 No Content` produces no output.

## Development

From the repository root, build the debug binary and add it to your `PATH` for the current shell:

```sh
cargo build --locked
export PATH="$PWD/target/debug:$PATH"
p2me --help
```

[CI](.github/workflows/ci.yml) checks formatting, runs Clippy, and runs tests on Linux for pull requests and pushes to `main`. To test the CLI against a local HTTP server, run:

```sh
cargo test --locked --all-targets --features test-endpoint
```

The local endpoint override works only in debug builds with the `test-endpoint` feature.

## Releases

[Changenote](https://github.com/changenote/changenote) uses [`.changenote/`](.changenote/) to manage a release pull request.

The [release workflow](.github/workflows/release.yml) pins the Changenote Action and CLI. Update both pins when a new CLI version requires a newer Action.

Changenote uses the `PAT_TOKEN` Actions secret when set. Use a fine-grained personal access token limited to this repository. Set **Contents** and **Pull requests** to **Read and write**.

If `PAT_TOKEN` is not set, Changenote uses the built-in `GITHUB_TOKEN`. Allow GitHub Actions to create pull requests in the repository's Actions settings. When Changenote creates or updates a release pull request with this token, the CI runs for that pull request wait for approval. A user with write access must select **Approve workflows to run** in the pull request's merge box. See [GitHub's `GITHUB_TOKEN` documentation](https://docs.github.com/en/actions/concepts/security/github_token#when-github_token-triggers-workflow-runs).

On pushes to `main`, the workflow also publishes Git tags and GitHub Releases for verified releases. Manual runs only manage the release pull request. The workflow does not upload packages to registries.

### Build for PyPI

On Linux, `uv build` can produce a wheel tagged `linux_x86_64`, which PyPI rejects. Ask Maturin to check PyPI compatibility and write the files to a separate directory:

```sh
uv build --clear --out-dir dist/pypi --config-setting 'build-args=--compatibility pypi'
```

When ready to publish, pass only the files from `dist/pypi/` so `uv publish` does not pick up an older wheel from `dist/`:

```sh
uv publish dist/pypi/*.whl dist/pypi/*.tar.gz
```
