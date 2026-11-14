# AGENTS.md: InstanceDeck

You are building **InstanceDeck**, a multi-instance app launcher for Windows, plus its GitHub Pages landing site.

## Read first
1. `HANDOVER.md`: the full spec. It is the source of truth for behavior.
2. `design/README.md`: the Figma frames (node IDs and links), PNG exports and build notes. **The Figma frames are the authority for every visual and for the final copy.** If you have the Figma MCP server, read the frames with `get_design_context` and `get_variable_defs`; otherwise use `design/*.png`.
3. `design/tokens.json`: generate CSS variables and TS constants from this file. Never hand-type hex values.
4. `DECISIONS.md`: any decision recorded there overrides `HANDOVER.md`. Create it on first run.
5. `PROGRESS.md`: what has been done so far. Create it on first run, and log side-by-side screenshots (Figma | built) for every screen and section you build.

## How to work
- Follow the phases in `HANDOVER.md` §11 in order. Do not start a phase until the previous phase's "Done when" criteria pass.
- At the end of every phase:
  - Update `PROGRESS.md` with what shipped, test output, screenshots or Storybook links, and known issues.
  - Commit `phase N: <summary>`.
- When the spec is silent or ambiguous, choose the most conservative option, log it in `DECISIONS.md` (date, context, choice, alternatives), and continue. Do not stop to ask.
- Use the latest stable versions of Tauri 2, React, Astro, Storybook and GitHub Actions. Pin exact versions in lockfiles.

## Hard rules (never break)
1. **Never modify, patch, copy or repackage a target app's binaries.** Isolation comes only from launch arguments, environment variables and data paths.
2. **Never read, parse, decrypt or transmit credentials.** That covers cookies, `auth.json`, tokens and DPAPI blobs. File metadata (exists, size, mtime, rename) is allowed.
3. **Never hard-delete user data.**
   - Use rename or move.
   - Deletion goes to the Recycle Bin only, after explicit confirmation.
   - A stash is never removed before it has been restored.
   - Backups and stashes are never auto-pruned.
4. **No DLL injection, no API hooking inside other processes, no telemetry, no network calls** except the user-enabled update check.
5. **Every UI component and screen state gets a Storybook story.** The component library in `packages/ui` is mandatory, including for the marketing site.
6. **Design taste:**
   - Flat colours only. No `box-shadow` and no gradients.
   - No outlined boxes or strong strokes; the focus ring is the only exception.
   - Ample whitespace, visuals over text.
   - Never draw third-party logos.
7. **Every page of the site has the footer credit** linking to https://tanziro.com and the non-affiliation line (`HANDOVER.md` §1.3, §8.4).
8. **No fabricated content.** No fake testimonials, ratings, download counts, customer logos or metrics.

## Commands (keep this list current)
- `pnpm i`: install everything
- `pnpm dev:desktop`: Tauri app in dev
- `pnpm storybook`: component library
- `pnpm dev:site`: Astro site
- `pnpm test`: all JS tests · `cargo test --workspace`: Rust tests
- `pnpm lint` · `cargo clippy --workspace -- -D warnings`
- `pnpm typecheck`: TypeScript and Astro checks
- `pnpm build:storybook`: static component documentation
- `pnpm test:smoke`: desktop, site and all Storybook browser checks
- `pnpm build:desktop`: native Windows build (no installer until P7)
- `pnpm exec playwright install chromium`: install the browser used by tests

## Windows specifics
- Windows-only code lives in `crates/ideck-win` behind traits from `crates/ideck-core`, with fakes for tests.
- Test paths with spaces and non-ASCII characters.
- Never hard-code `WindowsApps` paths. Resolve packages at runtime (`HANDOVER.md` §4.4).
