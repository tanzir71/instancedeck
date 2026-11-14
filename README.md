# Run multiple instances of your desktop apps on Windows

InstanceDeck is a multi-instance app launcher for Windows. It launches official
apps with separate arguments, environment variables and profile data paths.

**Status:** phase 0 scaffold under active development. Profile launching is not
implemented yet. See [PROGRESS.md](PROGRESS.md) for evidence and remaining
gates.

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

No target app binaries or credential files are read, copied or changed by this
scaffold. InstanceDeck is not a security sandbox. The app makes no network
calls.

## License and attribution

MIT © 2026 Tanzir Rahman. Designed and built by
[Tanzir Rahman · tanziro.com](https://tanziro.com).

InstanceDeck is an independent open-source project. It is not affiliated with,
endorsed by or sponsored by Anthropic, OpenAI, Microsoft or any other app
vendor. Claude is a trademark of Anthropic, PBC. Codex and ChatGPT are
trademarks of OpenAI. All other trademarks belong to their owners.
