# p2me

`p2me` is a command-line client for [Prove2me](https://prove2.me/). Find a theorem, submit a Lean proof or disproof, and check the result. Prove2me verifies submissions on its servers, so you do not need Lean installed to use the CLI.

## Quick start

1. Build the CLI from the repository root and put the debug binary on your `PATH` for the current shell:

   ```sh
   cargo build --locked
   export PATH="$PWD/target/debug:$PATH"
   p2me --help
   ```

2. Get an API key from your Prove2me account menu. Set `PROVE2ME_API_KEY` in your environment, or put this line in a regular `.env` file in the directory where you run `p2me`:

   ```dotenv
   PROVE2ME_API_KEY=your-key
   ```

   Use an unquoted value with no spaces or inline comment. The CLI does not expand variables or look in parent directories. The environment variable takes precedence even when it is empty. Do not commit `.env` or use one from an untrusted directory.

3. Search for a theorem, then replace `THEOREM_ID` with an ID from the search results:

   ```sh
   p2me theorems search 'poincare'
   p2me theorems get THEOREM_ID
   ```

4. Submit a Lean file for that theorem. Uploads must be regular files.

   ```sh
   p2me verify submit THEOREM_ID solution.lean
   ```

   Submissions default to `--proof-type prove`. To submit a disproof, use `--proof-type disprove`. `--explanation` is optional for either mode.

   ```sh
   p2me verify submit THEOREM_ID counterexample.lean --proof-type disprove --explanation 'A counterexample.'
   ```

5. Replace `SUBMISSION_ID` with the `submission_id` from the upload response:

   ```sh
   p2me verify status SUBMISSION_ID
   ```

   The upload command returns without waiting for verification and does not retry. If the connection fails after the upload starts, check your Prove2me submission history before submitting again.

## Read the output

Commands that return data write one indented JSON result to standard output. Field names match the API: the upload response uses `submission_id`, while the status response uses `id`.

A status lookup can exit `0` even when the proof has not been accepted. Check the `status` field for values such as `PENDING` or `WA`. `SKETCH_ACCEPTED` means Prove2me accepted a reduction to claims that still need proofs, not a full proof. Command failures exit nonzero and write diagnostics to standard error.

See the [Prove2me API and CLI coverage matrix](docs/prove2me-api-coverage.md) for request parameters and supported commands.

## Development

[CI](.github/workflows/ci.yml) checks formatting, runs Clippy, and runs tests on Linux for pull requests and pushes to `main`. To test the built CLI against a local HTTP server, run:

```sh
cargo test --locked --all-targets --features test-endpoint
```

The local endpoint override requires both a debug build and the `test-endpoint` feature. Release builds do not support it.

## Releases

[Changenote](https://github.com/changenote/changenote) uses [`.changenote/`](.changenote/) to manage a release pull request. In the repository's Actions settings, allow GitHub Actions to create pull requests.

The [release workflow](.github/workflows/release.yml) pins both the Changenote Action and CLI. Update them together when a new CLI version requires a newer Action.

On pushes to `main`, the workflow also publishes Git tags and GitHub Releases for verified releases. Manual runs only manage the release pull request. The workflow needs no custom secrets and does not upload packages to registries.

### Build for PyPI

On Linux, a plain `uv build` can produce a wheel tagged `linux_x86_64`, which PyPI rejects. Ask Maturin to check PyPI compatibility and write the distributions to a separate directory:

```sh
uv build --clear --out-dir dist/pypi --config-setting 'build-args=--compatibility pypi'
```

When ready to publish, pass only the files from `dist/pypi/` so `uv publish` does not pick up an older wheel from `dist/`:

```sh
uv publish dist/pypi/*.whl dist/pypi/*.tar.gz
```
