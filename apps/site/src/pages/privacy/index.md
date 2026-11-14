---
layout: ../../layouts/ArticleLayout.astro
title: Privacy – InstanceDeck
description:
  InstanceDeck has no telemetry, accounts or credential access. Profile data
  stays on your PC. Learn what local diagnostics contain.
heading: Your profiles stay on your PC
name: Privacy
intro: No analytics, no InstanceDeck account and no credential reading.
---

## Desktop data

InstanceDeck stores its own profile names, IDs, paths, colors, preferences and
launch logs locally. Profiles default to `%LOCALAPPDATA%\InstanceDeck\Profiles`;
you can choose another root. Settings and metadata updates retain backups.
Backups and sign-in stashes are never automatically pruned.

The launcher uses process metadata and file metadata to discover apps, track
profiles and identify supported data layouts. It never reads, parses, decrypts
or transmits app cookies, authentication files, session tokens or DPAPI blobs.

Your target apps have their own privacy policies and network activity.
InstanceDeck's local-only design does not change those apps' behavior.

## Diagnostics are local

Diagnostics show InstanceDeck's launch settings, discovered app versions,
process status and structured launcher logs. They do not include target app file
contents. You can redact paths before copying a report. Review a report before
posting it publicly.

## Network activity

InstanceDeck has no telemetry or background analytics. Its only intended network
feature is a user-enabled update check against the project's public release
feed. Update checking is unavailable in the current development build; enabling
the preference does not send requests yet.

## This website

The site uses local assets and has no third-party analytics, advertising scripts
or tracking cookies. It is hosted on GitHub Pages; GitHub processes hosting
requests under its
[privacy statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement).
External links, including downloads and sign-in pages, are subject to their
hosts' policies.

## Questions

Contact the maintainer through the
[project repository](https://github.com/tanzir71/instancedeck). Do not post
credentials or private app data in an issue. See [Security](../security/) for
reporting a vulnerability.
