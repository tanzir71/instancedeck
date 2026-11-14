# Progress

## P0 — Scaffold and tooling (complete)

Started 2026-11-14 on Windows 11 25H2, build 26200, x64. Initial workspace:
handover, AGENTS.md and Figma exports only; no Git repository.

- Read HANDOVER.md, design/README.md and tokens.json.
- Read live Figma components (`151:2297`), Deck (`155:2321`) and foundation
  variables (`154:2306`), session `79905340-1e2a-4935-b073-fed9959be449`.
- Node 24.22.2, pnpm 10.33.2, Rust 1.97.0; VS 2022 C++ tools available.

Implemented:

- pnpm/Turborepo workspaces, exact dependency pins and lockfiles; Rust workspace
  with platform/core/CLI boundaries and Tauri 2 Windows shell.
- Style Dictionary generates CSS variables and TypeScript constants directly
  from `design/tokens.json`; fonts and SVG assets are local.
- Shared Button variants, Logo, empty Deck and Astro scaffold/footer; all
  shipped components/states have Storybook stories.
- ESLint, Prettier, stylelint flat-color/no-shadow/no-stroke rules, rustfmt,
  warnings-denied clippy and a Windows CI workflow.
- `pnpm test` now includes contrast checks, both application smoke tests and
  every Storybook interaction/accessibility test. Browser tests check that site
  SVGs really load; visual inspection found and corrected an Astro SVG import
  issue.

Local evidence (2026-11-14):

- All three required development commands started on Windows 11: native WebView2
  empty Deck, Storybook at port 6006, Astro under `/instancedeck/` at port 4321.
- `pnpm typecheck`, `pnpm lint`, `pnpm build`, `pnpm build:storybook`: passed.
- `cargo test --workspace`: passed (scaffold has no Rust domain tests yet).
- `cargo clippy --workspace -- -D warnings`: passed.
- `pnpm build:desktop`: passed; produced an x64 native executable.
- `pnpm test`: all 4 browser tests passed, including all 20 Storybook stories
  across 5 suites, the corrected site SVGs and all eight OnColor palette cases.
- [Windows CI](https://github.com/tanzir71/instancedeck/actions/runs/43575723466)
  passed all steps, including native x64 release compilation (9m28s).
- [Final tooling validation](https://github.com/tanzir71/instancedeck/actions/runs/43576518592)
  passed every step for the path-safe generator and stricter CSS lint rules.
  Negative local fixtures also confirmed logical borders, gradients and shadows
  are rejected. P0's required development commands and Windows CI gate passed.

Visual evidence (Figma | built):

| Reference                                       | Built                                               |
| ----------------------------------------------- | --------------------------------------------------- |
| ![Figma Deck](design/20-app-deck@2x.png)        | ![Empty Deck shell](artifacts/p0/desktop-empty.png) |
| ![Figma components](design/01-components.png)   | ![Primary Button](artifacts/p0/button.png)          |
| ![Figma foundations](design/00-foundations.png) | ![Generated foundations](artifacts/p0/tokens.png)   |

[Native Windows screenshot](artifacts/p0/native-empty.jpg) ·
[Site scaffold screenshot](artifacts/p0/site-scaffold.png) ·
[Storybook Button](http://127.0.0.1:6006/?path=/story/components-button--primary)
·
[All palette cases](http://127.0.0.1:6006/?path=/story/components-button--profile-palette)

Known limits: this is the empty scaffold, with native window decoration. The
populated Deck and custom title bar are P2; the complete landing site is P6. One
evidence-backed accessibility deviation from the Figma OnColor translucency is
recorded in `DECISIONS.md`. P1–P7 have not started.
