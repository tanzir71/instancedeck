# Run multiple instances of your desktop apps on Windows

InstanceDeck is a multi-instance app launcher for Windows. It launches official
apps with separate arguments, environment variables and profile data paths.

**Status:** the scaffold and launch engine/CLI are verified on Windows 11. The
desktop UI remains a preview. See [PROGRESS.md](PROGRESS.md) for evidence and
remaining gates.

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
```

Target app binaries stay untouched; credentials are never inspected.
InstanceDeck is not a security sandbox. The app makes no network calls.

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

## License and attribution

MIT © 2026 Tanzir Rahman. Designed and built by
[Tanzir Rahman · tanziro.com](https://tanziro.com).

InstanceDeck is an independent open-source project. It is not affiliated with,
endorsed by or sponsored by Anthropic, OpenAI, Microsoft or any other app
vendor. Claude is a trademark of Anthropic, PBC. Codex and ChatGPT are
trademarks of OpenAI. All other trademarks belong to their owners.
