# Decisions

## 2026-11-14 — OnColor contrast correction

Context: axe reported color-contrast failure for white button text on a teal
profile with the Figma `paper-16` button fill. Calculated contrast is 3.57:1;
all eight profile fills fall below 4.5:1 at that translucency. Neither white nor
ink text reaches 4.5:1 on the resulting iris/plum fill. Choice: mix `paper-16`
at 12.5% with transparent for an effective 2% white fill. This is the smallest
visual change that preserves the white label, flat profile palette, control size
and WCAG AA across every swatch. Keep this explicit design deviation for review.
Alternatives: dark labels (still fail iris/plum), changing the palette, or
enlarging labels enough to qualify as large text.

## 2026-11-14 — Compatible TypeScript and font packaging

Context: the stable TypeScript 7 tag exceeds typescript-eslint's supported range
(`>=4.8.4 <6.1.0`). Fontsource variable packages expose font files, not
`latin.css`. Choice: pin TypeScript 6.0.3 (the newest supported 6.x), explicitly
declare the WASM runtime's required emnapi peers, and self-host Latin WOFF2
files under the font-family names in tokens.json. Alternatives: suppressing peer
warnings or loading unsupported subpaths. React, Astro, Storybook and Tauri 2
remain at their latest stable versions resolved on this date.

## 2026-11-14 — Product defaults

Context: HANDOVER.md §13.2 delegates these choices with defaults. Choice:
InstanceDeck; MIT and fully free; GitHub Pages at
`https://tanzir71.github.io/instancedeck/`; no usage display; Source Serif 4,
IBM Plex Sans and IBM Plex Mono. Alternatives: the named alternate brands, paid
licensing, custom domain, usage reporting and Public Sans.

## 2026-11-14 — Ordered acceptance gates

Context: this workspace starts with only the handover and design assets. Choice:
implement and verify P0 first, including actual Windows CI, before P1. P6 may
start after P0 as explicitly allowed by §11. A scaffold is not a completed
product. Never mark manual account isolation or signing verified without
evidence. Alternative: building later phases before their gates pass.

## 2026-11-14 — Stable dependency resolution

Context: the latest Tauri release on crates.io is a v3 alpha; the stack requires
Tauri 2 stable. Choice: select the newest non-prerelease v2, and resolve the
other tools from their stable registry tags. Pin package manifests and both
lockfiles. Alternatives: adopting prereleases or using stale handover versions.

## 2026-11-14 — Phase 0 preview

Context: profile operations belong to P1–P2, after the scaffold gate. Choice:
the initial desktop opens the empty Deck with an explicitly labelled preview
state; no fake profiles, status counts or operational launch buttons. The site
initially contains a scaffold page with the required footer. Full landing
sections belong to P6. Alternative: presenting demo data as real state.
