# Run multiple instances of your desktop apps on Windows

InstanceDeck is a multi-instance app launcher for Windows. It launches official
apps with separate arguments, environment variables and profile data paths.

**Status:** the launch engine/CLI are verified on Windows 11. The desktop UI is
implemented with native keyboard/tray QA deferred to the user. Journaled Claude
sign-in and recovery pass synthetic crash tests; actual account verification is
pending. Windows shortcuts, window identity, hotkeys, the switcher and autostart
are implemented; native pinning verification remains deferred. Eight built-in
recipes, a custom app editor, metadata import/export, opt-in Codex sessions
sharing and dark theme are implemented. All recipes remain Community pending
manual account verification. The
[Astro site](https://tanzir71.github.io/instancedeck/) is live, with 25 pages
and live Lighthouse scores of 97–100 across all categories. See
[PROGRESS.md](PROGRESS.md) for evidence and remaining gates.

The desktop uses Tauri 2, React and TypeScript. The static Astro site and
desktop share `packages/ui`; all visual tokens are generated from
`design/tokens.json`.

## Development

Windows 11, Node 24 LTS, pnpm 10.33.2, Rust 1.97.0, Visual Studio C++ Build
Tools and Microsoft Edge WebView2 are required.

```powershell
pnpm i
pnpm dev:desktop
pnpm storybook
pnpm dev:site
```

The three development commands run separately. The site is served at
`http://127.0.0.1:4321/instancedeck/`.

```powershell
pnpm typecheck
pnpm lint
pnpm test
cargo test --workspace
cargo clippy --workspace -- -D warnings
pnpm build:storybook
pnpm exec playwright install chromium
pnpm test:smoke
pnpm build:desktop
pnpm --filter @instancedeck/site build
pnpm --filter @instancedeck/site test:lighthouse
```

Target app binaries stay untouched; credentials are never inspected.
InstanceDeck is not a security sandbox. The app makes no network calls. The site
build fetches public GitHub release metadata. Browser tests serve the production
site at port 4322; Lighthouse uses a separate foreground preview.

## Engine CLI

```powershell
cargo run -p ideck-cli -- apps
cargo run -p ideck-cli -- profiles create claude-desktop Work --color iris
cargo run -p ideck-cli -- profiles list
cargo run -p ideck-cli -- launch <profile-id>
cargo run -p ideck-cli -- close <profile-id>
cargo run -p ideck-cli -- doctor
```

`--state-dir` and `--profiles-root` accept absolute paths, including another
drive and Unicode names. `plan <id>` shows the resolved command without
launching it. `profiles remove <id>` only removes deck metadata and keeps data.
Force close requires the profile name after an eight-second graceful attempt.
Recipe status stays Community until the full manual account checklist has been
run.

Claude sign-in recovery is available in the desktop assistant and CLI:

```powershell
cargo run -p ideck-cli -- sign-in list
cargo run -p ideck-cli -- sign-in start <profile-id> --close-main
cargo run -p ideck-cli -- sign-in finish <journal-id>
cargo run -p ideck-cli -- sign-in restore <journal-id>
```

`--close-main` explicitly permits graceful closure of your main Claude. Save
your work first. An unfinished journal blocks ordinary Claude launches until it
is resumed or restored. Captures, previous profile data and journal history are
retained; nothing is automatically pruned. The metadata hint does not establish
account identity. Confirm the visible account inside the official app.

## License and attribution

MIT © 2026 Tanzir Rahman. Designed and built by
[Tanzir Rahman · tanziro.com](https://tanziro.com).

InstanceDeck is an independent open-source project. It is not affiliated with,
endorsed by or sponsored by Anthropic, OpenAI, Microsoft or any other app
vendor. Claude is a trademark of Anthropic, PBC. Codex and ChatGPT are
trademarks of OpenAI. All other trademarks belong to their owners.
