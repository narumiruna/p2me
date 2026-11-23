# p2me

GitHub Actions checks formatting, runs Clippy, and runs tests on Linux for pull requests and pushes to `main`.

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
