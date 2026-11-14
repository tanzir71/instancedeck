# InstanceDeck: design handover (Figma)

**Figma file:** Project Priority – Tack
**Page:** `InstanceDeck · landing + app · 14 Nov`
https://www.figma.com/design/8o7PprHCNdQxS3WvHzv62e/Project-Priority---Tack?node-id=150-2297

These Figma frames are the **design authority** for the build. Where a frame and `HANDOVER.md` §8 disagree, the frame wins. Where the frame's text differs from the copy deck in `HANDOVER.md` §9.5–9.6, the frame's text is the final copy.

## How to read the design

1. **If your environment has the Figma MCP server:** call `get_design_context` (or `get_metadata`, then `get_design_context`) on the node IDs below.
   - Call `get_variable_defs` for tokens.
   - Treat the output as a reference to adapt into `packages/ui` components; never paste generated code wholesale.
2. **Otherwise:** use the PNG exports in this folder plus `tokens.json`. Both match the Figma file as of 14 Nov 2026.

Everything in the file is editable layers built from components. Colours are bound to the variable collection **`InstanceDeck · Color`** (`ui/*`, `profile/*`, `alpha/*`), and all text uses the **`InstanceDeck/*`** text styles. `tokens.json` mirrors both.

## Frames

| Frame | Node | Link | Export |
|---|---|---|---|
| 00 · Foundations (taste rules, colour, type, space) | `154:2306` | [open](https://www.figma.com/design/8o7PprHCNdQxS3WvHzv62e/Project-Priority---Tack?node-id=154-2306) | `00-foundations.png` |
| 01 · Components | `151:2297` | [open](https://www.figma.com/design/8o7PprHCNdQxS3WvHzv62e/Project-Priority---Tack?node-id=151-2297) | `01-components.png` |
| 10 · Landing — Desktop 1440 | `157:2410` | [open](https://www.figma.com/design/8o7PprHCNdQxS3WvHzv62e/Project-Priority---Tack?node-id=157-2410) | `10-landing-desktop-overview.png` (downscaled overview), full-res sections in `landing-sections/` |
| 11 · Landing — Mobile 390 | `160:2632` | [open](https://www.figma.com/design/8o7PprHCNdQxS3WvHzv62e/Project-Priority---Tack?node-id=160-2632) | `11-landing-mobile-390@2x.png` |
| 20 · App — Deck | `155:2321` | [open](https://www.figma.com/design/8o7PprHCNdQxS3WvHzv62e/Project-Priority---Tack?node-id=155-2321) | `20-app-deck@2x.png` |
| 21 · App — New profile (step 2 of 3) | `161:2709` (dialog `161:2791`) | [open](https://www.figma.com/design/8o7PprHCNdQxS3WvHzv62e/Project-Priority---Tack?node-id=161-2709) | `21-app-new-profile@2x.png` |
| 22 · App — Claude sign-in assistant | `161:2857` (dialog `161:2939`) | [open](https://www.figma.com/design/8o7PprHCNdQxS3WvHzv62e/Project-Priority---Tack?node-id=161-2857) | `22-app-claude-sign-in@2x.png` |

### Desktop landing sections (top to bottom)

| # | Section | Node | Band colour | Export |
|---|---|---|---|---|
| 1 | Header | `157:2411` | paper | `landing-sections/01-header@2x.png` |
| 2 | Hero (copy + fanned deck) | `157:2428` | paper / canvas field | `02-hero@2x.png` |
| 3 | Works with | `157:2497` | canvas | `03-works-with@2x.png` |
| 4 | Pillars | `157:2536` | paper | `04-pillars@2x.png` |
| 5 | How it works (steps + product shot) | `158:2481` | paper, accent field | `05-how-it-works@2x.png` |
| 6 | Use cases | `158:2582` | canvas | `06-use-cases@2x.png` |
| 7 | Claude / Codex spotlight | `158:2612` | paper, canvas blocks | `07-spotlight-claude-codex@2x.png` |
| 8 | Security | `159:2591` | deep | `08-security@2x.png` |
| 9 | FAQ | `159:2627` | paper | `09-faq@2x.png` |
| 10 | CTA | `159:2673` | accent | `10-cta@2x.png` |
| 11 | Footer, with the tanziro.com credit (`159:2717`) | `159:2679` | deep | `11-footer@2x.png` |

## Components (map 1:1 to `packages/ui`)

| Figma component | Node | Variants and properties | React component |
|---|---|---|---|
| Button | `151:2315` | Kind = Primary · Secondary · Inverse · OnColor; Size = L (52) · M (40); `Label` | `<Button kind size>` |
| MonogramTile | `151:2334` | Tone = Light · Neutral · Ink; Size = L 44 · M 32 · S 24; `Letters` | `<MonogramTile>` (stands in for vendor logos) |
| Eyebrow | `151:2341` | Tone = Default · OnDark; `Text` | `<Eyebrow>` |
| Badge | `151:2348` | Kind = Verified · Community · Experimental | `<Badge>` (recipe status) |
| InstanceCard | `151:2371` | Status = Running · Idle; `Name`, `App`, `Path`; fill = any `profile/*` | `<InstanceCard>` (marketing deck) |
| ProfileCard | `151:2400` | Status = Running (Focus + Close) · Idle (Launch); `Name`, `Detail`; fill = `profile/*` | `<ProfileCard>` (desktop app) |
| MainInstanceCard | `151:2401` | none | `<MainInstanceCard>` |
| AddProfileCard | `151:2414` | none | `<AddProfileCard>` |
| FAQItem | `151:2430` | State = Closed · Open; `Question`, `Answer` | `<FAQItem>` on native `<details>` |
| Logo | `155:2320` | Tone = OnLight · OnDark | `<Logo>`; SVGs in `assets/` |

## Build notes Codex must follow

- **Hero deck geometry** (desktop):
  - Five `InstanceCard`s, each 240×320, stacked at the same centre point: `left: 346px; top: 410px` inside the 680×780 canvas field, then translated by `-50%`.
  - Each card gets `transform-origin: 50% calc(50% + 1000px)` and `rotate(-10deg | -5deg | 0 | 5deg | 10deg)`.
  - DOM order runs left to right, so the right-most card is on top.
  - Colours, back to front: terracotta "Personal · Claude Desktop" (idle), ochre "QA · Visual Studio Code" (running), teal "Client — Acme · Codex" (running), plum "Research · Claude Desktop" (idle), deep "Work · Claude Desktop" (running).
  - On load, animate once from `rotate(0)` to the final fan (240 ms, ease-out, 40 ms stagger). No animation under `prefers-reduced-motion`.
  - The field clips overflow.
- **Hero deck (mobile 390):** cards 180×236 with 18 px padding, `transform-origin: 50% calc(50% + 600px)`, angles ±4°/±8°, centre `(195px, 248px)` in a 390×420 canvas field.
- **Product shot** (How it works): until the real app exists, use `20-app-deck@2x.png` at 87.5% (1120×700) on a full-width `ui/accent` field, 72 px top padding, bottom-aligned and cropped by the field (it bleeds off the bottom edge). Replace it with a real app screenshot once P2 ships.
- **Translucency:** the profile cards' button fills, secondary text and status dots use the `alpha/*` tokens (see `tokens.json`). Never apply CSS `opacity` to a whole component.
- **Icons:** the nav glyphs in `assets/icons/` are filled geometric shapes on `currentColor`. For anything else, use Fluent UI System Icons **Filled** only. No stroke icons.
- **Lines:** the only lines in the whole design are the 1 px `canvas-2` rule under each FAQ row and the 4 px stepper bars. Both are fills, not borders.
- **Footer credit:** "Designed and built by Tanzir Rahman · tanziro.com". Both the name and "tanziro.com" link to https://tanziro.com and are underlined. It appears on desktop and mobile, on every page.
- **"Signed releases"** in the security band renders only when `SIGNING_ENABLED=true`.

## Copy that exists only in Figma (final; add it to `apps/site` content)

- **Pillars:**
  - Eyebrow: "Why InstanceDeck"
  - H2: "Separate accounts. Same desktop."
  - Intro: "InstanceDeck gives every account its own profile, so the apps you rely on behave as if each one were the only one running."
  - Visual 3 caption: "0 — telemetry events, accounts or tokens read".
- **How it works:** the step bodies are extended. Step 1: "…and shows which ones it has verified." Step 2: "…choose where its data lives — on this drive or another." Step 3: "…Pin it, and the pin reopens that profile."
- **Use cases:**
  - Eyebrow: "Use cases"
  - Intro: "If you hold more than one account for the same app, InstanceDeck keeps each one open, signed in and separate."
- **Spotlight:** Codex body ends with "— and you can check that every profile shows its own account." Link labels: "Read the Claude Desktop guide →", "Read the Codex guide →".
- **Security:**
  - Body: "Profiles are isolation, not a sandbox. The threat model, every launch flag and the source are public, so your security team can check each claim."
  - Fact sub-lines: "MIT licensed, built in public" · "No analytics, no accounts" · "Never reads cookies or tokens" · "Official binaries, untouched" · "Deleted profiles stay recoverable".
- **FAQ** left column: "Questions, answered." / "Plain answers about accounts, data and trust." / "All questions →".
- **Footer:**
  - Tagline: "The multi-instance app launcher for Windows. Free and open source."
  - Columns:
    - Product: Download, How it works, Apps, Changelog
    - Resources: Claude Desktop guide, Codex guide, Docs, FAQ
    - Project: GitHub, Security, Code signing policy, Contributing
    - Legal: Privacy, License (MIT), Trademarks
- **App:**
  - Deck header meta: "7 profiles · 4 running · profiles on D:\Profiles"
  - New-profile preview note: "Shortcuts, hotkey and the taskbar icon use this colour. You can change it later."
  - Sign-in step copy: as shown in frame 22.

## Files in this folder

```
design/
├─ README.md                      ← this file
├─ tokens.json                    ← colour (ui / profile / alpha), type, space, radius, layout, motion
├─ 00-foundations.png
├─ 01-components.png
├─ 10-landing-desktop-overview.png
├─ 11-landing-mobile-390@2x.png
├─ 20-app-deck@2x.png
├─ 21-app-new-profile@2x.png
├─ 22-app-claude-sign-in@2x.png
├─ landing-sections/01-header@2x.png … 11-footer@2x.png
└─ assets/
   ├─ logo-mark.svg, logo-mark-on-dark.svg   (hand-authored, exact token colours)
   ├─ logo-lockups-figma.svg                 (Figma export, wordmark outlined)
   └─ icons/deck.svg, apps.svg, diagnostics.svg, settings.svg
```
