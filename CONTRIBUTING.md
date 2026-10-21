# Contributing

## Branch workflow

Do not commit directly to `main`.

1. Create a branch from the latest `main`:

   ```sh
   git switch main
   git pull --ff-only origin main
   git switch -c feature/short-description
   ```

2. Make the change and run the local checks:

   ```sh
   cargo fmt --all
   cargo test
   npm --prefix frontend ci
   npm --prefix frontend run build
   cargo check --manifest-path src-tauri/Cargo.toml
   ```

3. Push the branch and open a pull request targeting `main`.
4. Merge only after CI passes and the change has been reviewed.

Branch names should use `feature/`, `fix/`, `chore/`, or `docs/`.

## Releases and updates

Releases are created by pushing a version tag after the pull request has been
merged. The tag version must match the `version` in
`src-tauri/tauri.conf.json`:

```sh
git switch main
git pull --ff-only origin main
git tag v0.1.0
git push origin v0.1.0
```

For a beta release, use a hyphenated tag:

```sh
git tag v0.1.0-beta.1
git push origin v0.1.0-beta.1
```

The release workflow builds and publishes downloadable installers for:

- macOS Apple Silicon
- macOS Intel
- Windows x64
- Linux x64

The GitHub Release is published automatically. A tag such as `v0.1.0-beta.1`
is marked as a prerelease; a tag such as `v0.1.0` is marked as a stable
release.

The Tauri updater reads its signed `latest.json` manifest from the latest
GitHub Release. Keep the updater signing private key only in GitHub Actions
secrets; never commit it.

Before the first release, replace the placeholder public key in
`src-tauri/tauri.conf.json` and configure:

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

The `release.yml` workflow currently marks all tagged builds as prereleases.
Promote a tested release in GitHub after validation.
