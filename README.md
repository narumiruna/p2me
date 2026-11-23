# p2me

`p2me` is a command-line client for Prove2me. Search for theorems, submit a Lean proof or disproof, and check verification results without building HTTP requests. You do not need Lean installed to use the CLI.

## Usage

Build from the repository root with `cargo build --locked`. Run `target/debug/p2me --help` for the command list. The examples below assume `p2me` is on your `PATH`.

Get an API key from your Prove2me account menu. Set `PROVE2ME_API_KEY` in the process environment, or add `PROVE2ME_API_KEY=your-key` to `.env` in the directory you run the command from. The environment takes precedence, even if the variable is empty. In `.env`, use an unquoted value with no spaces or inline comment. The CLI does not expand variables or search parent directories. Do not commit the file or use one from an untrusted directory.

```sh
p2me theorems search 'poincare'
p2me theorems get "$theorem_id"
p2me verify submit "$theorem_id" solution.lean
p2me verify submit "$theorem_id" counterexample.lean --proof-type disprove --explanation 'A counterexample.'
p2me verify status "$submission_id"
```

Set `theorem_id` to an ID from a search result and submit the file that applies to that theorem. The default `--proof-type` is `prove`; `--explanation` is optional for either mode. Use the `submission_id` from the upload response on a later `verify status` call. Upload does not wait for verification or retry automatically. If a connection fails after upload starts, check your Prove2me submission history before uploading again.

Data commands write one indented JSON result to standard output. Fields keep their API names: the upload response uses `submission_id`, and the status response uses `id`. A successful lookup exits `0` even if `status` is `PENDING`, `WA`, or another non-acceptance result. `SKETCH_ACCEPTED` is an accepted reduction with unproved dependencies, not a fully proved theorem. Command failures exit nonzero and send diagnostics to standard error.

## Development

GitHub Actions checks formatting, runs Clippy, and runs tests on Linux for pull requests and pushes to `main`. Run `cargo test --locked --all-targets --features test-endpoint` to test the built CLI against a local HTTP server. The loopback override is only available in debug builds with this test feature, not in release builds.

[Changenote](https://github.com/changenote/changenote) uses [`.changenote/`](.changenote/) to manage a release pull request. On pushes to `main`, it also publishes tags and GitHub Releases for verified releases. Manual runs only manage the release pull request. In the repository's Actions settings, allow GitHub Actions to create pull requests. The GitHub release workflow needs no custom secrets and does not upload packages to registries.

## Build for PyPI

A plain `uv build` on Linux can produce a wheel tagged `linux_x86_64`, which PyPI rejects. Ask Maturin to check PyPI compatibility and write the upload files to a separate directory:

```sh
uv build --clear --out-dir dist/pypi --config-setting 'build-args=--compatibility pypi'
```

When ready to publish, pass only those files to `uv publish` so it does not pick up an older wheel from `dist/`:

```sh
uv publish dist/pypi/*.whl dist/pypi/*.tar.gz
```
