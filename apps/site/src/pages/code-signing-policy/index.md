---
layout: ../../layouts/ArticleLayout.astro
title: Code Signing Policy – InstanceDeck
description:
  InstanceDeck releases are unsigned. Review the build source, SHA-256 checksums
  and Windows trust limitations before installing.
heading: Code signing policy
name: Code signing policy
intro: Releases are unsigned. No trusted-publisher certificate is claimed.
---

## Scope

InstanceDeck distributes unsigned Windows installers and portable packages from
its public GitHub Releases page. It does not use a self-signed certificate or
ask users to install a trusted-root certificate. Target app binaries are never
modified, copied or repackaged.

Release artifacts are built by the project's GitHub Actions workflow from a
source tag. Review the corresponding source and workflow before deciding to
install a build.

## Windows trust

Unsigned software can trigger SmartScreen or be blocked by an organization's
policy. A checksum does not establish a trusted Windows publisher. Do not
disable security protections to install an untrusted file. Microsoft's
[code signing guidance](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options)
explains the distinction between unsigned and trusted-publisher distribution.

## Verification

Published releases will include SHA-256 checksums for both architectures and
package formats. Compare a downloaded file against the checksum from the same
release. A matching checksum checks integrity; it is not a substitute for
trusting the source of the checksum.

There is no pending SignPath application or signing-approval dependency.

[Download status](../download/) ·
[Source repository](https://github.com/tanzir71/instancedeck) ·
[Security model](../security/)
