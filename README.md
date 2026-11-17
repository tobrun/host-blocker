# HostBlocker

Block websites on your own machine by managing `/etc/hosts` — with a simple
desktop GUI, one curated block list, and an admin prompt for every change.

<p align="center"><img src="app-icon.png" width="128" alt="HostBlocker icon"></p>

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/tobrun/host-blocker/main/install.sh | sh
```

- **macOS**: installs `HostBlocker.app` into `/Applications` (universal
  binary; `curl` downloads are not quarantined, so Gatekeeper stays quiet).
- **Linux**: installs the `.deb` via `apt` where `dpkg` exists, otherwise
  puts the AppImage in `~/.local/bin` with a desktop entry.
- Remove it again with `--uninstall`:

  ```sh
  curl -fsSL https://raw.githubusercontent.com/tobrun/host-blocker/main/install.sh | sh -s -- --uninstall
  ```

  Tip: open **Settings → Remove all entries** before uninstalling to clean
  `/etc/hosts` first.

Prebuilt assets live on the [releases page](https://github.com/tobrun/host-blocker/releases).

## What it does

- **Your sites**: paste any page URL (e.g. `https://www.reddit.com/r/all`)
  and HostBlocker blocks the whole site — each custom host becomes four
  hosts-file entries (`0.0.0.0 host`, `0.0.0.0 www.host`, `:: host`,
  `:: www.host`).
- **Popular lists**: import one curated baseline and keep a local cache you
  can update — [StevenBlack hosts](https://github.com/StevenBlack/hosts)
  (unified base, optionally extended with `fakenews`, `gambling`, `porn`,
  `social` categories), [OISD Small](https://oisd.nl/) (low false-positive
  ads/trackers), [AdAway](https://adaway.org/) (small, conservative) or
  [Blocklist Project's phishing
  list](https://github.com/blocklistproject/Lists) (security-focused).
  Only one baseline is active at a time; importing another one replaces it.
- **Modes**: *Active*, *Pause my list* (baseline keeps blocking) and
  *Pause all* — pausing restores your original hosts file, which is backed
  up once to `/etc/hosts.hostblocker.bak`.
- **Privileges**: every change to `/etc/hosts` goes through the system
  admin prompt (macOS `osascript`, Linux `pkexec`). Cancelling the prompt
  changes nothing. App data (state, baseline cache) lives in the usual
  per-user app directory and needs no elevation.

> **Note:** browsers with DNS-over-HTTPS enabled (Firefox, Chrome with
> secure DNS) may resolve names outside `/etc/hosts` and bypass blocks.

## Development

Requirements: [pnpm](https://pnpm.io), Rust (stable), and Tauri 2's Linux
build deps (`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, …).

```sh
pnpm install
pnpm tauri dev        # run the app
pnpm test             # frontend tests (Vitest + Testing Library)
pnpm lint && pnpm typecheck
cargo test --manifest-path src-tauri/Cargo.toml   # Rust unit + integration tests
```

Mutation testing (CI runs the same tools):

```sh
cargo install cargo-mutants
cargo mutants --manifest-path src-tauri/Cargo.toml   # Rust
pnpm test:mutation                                   # frontend (StrykerJS)
```

### Release

Push a `v*` tag — `.github/workflows/release.yml` builds a macOS universal
bundle (ad-hoc signed) and Linux `.deb` + `.AppImage` and publishes them to
GitHub Releases, where `install.sh` picks them up automatically.

## License

MIT
