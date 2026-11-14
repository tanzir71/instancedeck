---
layout: ../../layouts/ArticleLayout.astro
title: Documentation – InstanceDeck
description:
  Find InstanceDeck setup, profile isolation, app compatibility, account
  guidance, diagnostics and development documentation.
heading: InstanceDeck documentation
name: Docs
intro: Start with the isolation model, then read the guide for your app.
---

## Getting started

1. Install the official app you want to use. InstanceDeck does not distribute
   it.
2. Check its [compatibility entry](../apps/) and reported limitations.
3. Create a named profile in InstanceDeck and choose its parent data folder.
4. Launch it and sign in through the official app. Check which account it shows.

The current build is in development. Guided Claude sign-in has automated crash
coverage; real account verification remains pending. Per-profile taskbar
identity, custom recipes and update installation have not shipped yet. A
preference or Storybook view does not imply that its later-phase engine is
available.

## Guides

- [How profiles work](../how-it-works/)
- [Claude Desktop accounts](../claude-desktop-multiple-accounts/)
- [Codex accounts](../codex-multiple-accounts/)
- [App compatibility and launch settings](../apps/)
- [Questions about accounts, data and trust](../faq/)

## Troubleshooting

If an app is not found, install it for the current Windows user and refresh
Apps. If a profile opens the wrong account, stop using that profile and run its
manual account health check. Do not copy authentication files between profiles.

Diagnostics shows the resolved launch plan and recent local launcher logs. Use
its redaction option before sharing a report. An unknown Claude data layout must
stop the guided sign-in rather than guess a path.

## Development and review

[Source and build instructions](https://github.com/tanzir71/instancedeck) ·
[Storybook component library](../storybook/) · [Contributing](../contributing/)
· [Security](../security/)
