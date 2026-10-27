# Releasing Claude Sessions

How a version gets from `master` to installed apps. Maintainers only.

## How it works

Pushing a version tag (`v0.4.0`) runs [`.github/workflows/build.yml`](../.github/workflows/build.yml):

1. **Check the release** (`prepare`): every manifest holds the tag's version
   (`scripts/bump-version.mjs --check`), the updater public key is set in
   `src-tauri/tauri.conf.json`, and the `TAURI_SIGNING_PRIVATE_KEY` secret exists.
2. **Build** (`build`), one job per platform. Release builds merge
   `{ "bundle": { "createUpdaterArtifacts": true } }` into the config, so Tauri
   also signs the updater bundles with the updater key:

   | Platform          | Files                                                        | Updater bundle         |
   | ----------------- | ------------------------------------------------------------ | ---------------------- |
   | Windows x64       | `…-win-Setup.exe` (NSIS), `…-win-portable.exe`               | `…-win-Setup.exe`      |
   | macOS arm64 / x64 | `…-macOS-<arch>.dmg`, `…-macOS-<arch>.app.tar.gz`            | `….app.tar.gz`         |
   | Linux x64 / arm64 | `…-linux-<arch>.AppImage`                                    | `….AppImage`           |

3. **Publish** (`publish`): `scripts/updater-manifest.mjs` writes `latest.json`
   from the bundles and their `.sig` files (it fails if a platform is missing),
   then the job creates the GitHub Release as a **draft**, uploads every file,
   and only then publishes it. The release body is the `## [x.y.z]` section of
   `CHANGELOG.md`, followed by GitHub's generated notes.

Installed apps read
`https://github.com/oliviermattei/claude-sessions-gui/releases/latest/download/latest.json`
at startup and every 6 hours. When it announces a newer version, a toast offers
**Restart to update**: the app downloads the bundle, checks its signature
against the public key built into it, installs it and restarts.

- A pre-release tag (`v0.4.0-beta.1`) makes a pre-release, which is never the
  "latest" release, so installed apps are not offered it.
- The **portable** Windows exe and dev builds never update themselves.
- `requireSignedVersion` is on: an update is refused unless its signature was
  made for the version `latest.json` announces.

## One-time setup: the updater key

The updater only installs files signed with the key whose public half is built
into the app.

1. Generate the key pair, with a strong password, **outside** the repository:

   ```sh
   pnpm tauri signer generate -w ~/.tauri/claude-sessions.key
   ```

2. Replace `REPLACE_WITH_THE_UPDATER_PUBLIC_KEY` in `src-tauri/tauri.conf.json`
   (`plugins.updater.pubkey`) with the content of `~/.tauri/claude-sessions.key.pub`,
   and commit it. The release workflow fails while the placeholder is there.
3. Add the repository secrets (Settings → Secrets and variables → Actions):
   - `TAURI_SIGNING_PRIVATE_KEY`: the content of `~/.tauri/claude-sessions.key`;
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: its password.
4. Keep the private key and its password in a password manager. Without them no
   update can ever reach installed apps again: users would have to reinstall by
   hand. Never commit the private key.

## Making a release

The `create-release` Claude Code skill walks through these steps.

1. Write the changelog: rename `## [Unreleased]` to `## [0.4.0] - YYYY-MM-DD`,
   put a fresh empty `## [Unreleased]` above it, and update the compare links at
   the bottom. The heading must read `## [0.4.0]` exactly.
2. Set the version in every manifest, and commit both:

   ```sh
   node scripts/bump-version.mjs 0.4.0
   git commit -am "chore(release): 0.4.0"
   git push
   ```

   The script writes `package.json`, `src-tauri/tauri.conf.json`,
   `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock`. CI fails while they
   disagree.
3. Tag and push the tag:

   ```sh
   git tag v0.4.0
   git push origin v0.4.0
   ```

4. Follow the **Release** workflow in the Actions tab. Once it succeeds,
   installed apps offer the update at their next check.

If a published release is broken, publish a fixed version rather than deleting
it: apps that already updated only move forward. To check an update end to end,
install the previous version, then start it once the new release is published.
