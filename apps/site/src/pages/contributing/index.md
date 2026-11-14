---
layout: ../../layouts/ArticleLayout.astro
title: Contributing – InstanceDeck
description:
  Help improve InstanceDeck with reproducible app compatibility reports,
  accessible components and conservative data-safety changes.
heading: Contribute to InstanceDeck
name: Contributing
intro: Reproducible evidence matters more than a compatibility claim.
---

## Report app compatibility

Include the app version, Windows version, architecture, install type, launch
strategy and the visible result. Confirm account identity manually in the target
app. Never attach credentials, cookies, authentication files or private
conversations.

Community recipes stay Community until the required account and recovery checks
have evidence. A successful process launch alone does not prove account
isolation.

## Change the code

Read `AGENTS.md`, `HANDOVER.md`, `DECISIONS.md` and `PROGRESS.md` in the
[repository](https://github.com/tanzir71/instancedeck). Keep Windows code behind
the core traits, add fakes for behavior tests, and include paths with spaces and
non-ASCII characters.

All UI belongs in `packages/ui` with applicable Storybook states. Tokens come
from the design source. Flat colors, keyboard access and visible focus are
required.

Run `pnpm test`, `pnpm lint`, `pnpm typecheck`, `cargo test --workspace` and
`cargo clippy --workspace --all-targets -- -D warnings` for a relevant change.
Include screenshots and test evidence with your pull request.

## Data-safety boundaries

No credential reads, binary modifications, injection, telemetry or hard deletion
of user data. Use rename or move and retain backups. Do not “clean up” an
existing stash before it has been restored. For vulnerabilities, use the
[security reporting guidance](../security/).
