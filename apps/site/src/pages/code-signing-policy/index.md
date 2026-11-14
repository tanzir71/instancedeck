---
layout: ../../layouts/ArticleLayout.astro
title: Code Signing Policy – InstanceDeck
description:
  InstanceDeck's signing status, trusted build policy, release verification and
  planned SignPath Foundation application.
heading: Code signing policy
name: Code signing policy
intro: Signing approval is pending. Current development builds are unsigned.
---

## Scope

Only InstanceDeck's own Windows executables and installers built from this
public repository are eligible for signing. Target app binaries are never
modified, copied, repackaged or submitted for signing.

Signing must originate from the project's trusted GitHub Actions release
workflow, using a reviewed source tag. A local binary or an arbitrary uploaded
artifact must never enter that signing path. All project team members must use
multifactor authentication before a signing service is enabled.

## Approval status

The project plans to apply to SignPath Foundation after a public release. No
certificate or approval is claimed. The release workflow and site keep signing
disabled until approval and a successful trusted signing run are recorded.

If approved, the required credit is: “Free code signing provided by SignPath.io,
certificate by SignPath Foundation”. This is a conditional credit, not a
statement that the current build has been signed.

## Verification

Published releases will include SHA-256 checksums for both architectures and
package formats. Compare a downloaded file against the checksum from the same
release. A matching checksum checks integrity; it is not a substitute for
trusting the source of the checksum.

Windows may warn about new or unsigned software. Verify the release and
publisher information; do not bypass a security warning merely because a page
tells you to.

[Download status](../download/) ·
[Source repository](https://github.com/tanzir71/instancedeck) ·
[Security model](../security/)
