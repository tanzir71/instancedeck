# Unsigned release policy

InstanceDeck's Windows executables and NSIS installers are **unsigned**. No
trusted certificate, signing-service approval or future code-signing promise is
implied. The project does not ask users to install a self-signed root
certificate or disable Windows protection. SmartScreen or organizational policy
may warn or block execution.

Downloads come from
[this repository's Releases](https://github.com/tanzir71/instancedeck/releases).
Each draft/release includes x64 and ARM64 per-user NSIS installers, portable
archives, build provenance and SHA-256 checksums. Compare a downloaded file with
`Get-FileHash -Algorithm SHA256 -LiteralPath '.\your-file.exe'` and the matching
checksum entry before using it. Checksums alone are not a publisher identity.

Updater integrity signatures are separate from Windows Authenticode. A local
project key signs the exact CI-built installer and its app version. The public
key is embedded in the launcher; the updater rejects invalid signatures,
unsigned versions, mismatched versions and downloads outside this repository's
HTTPS release assets. The feed also has a detached integrity signature for
offline verification. The launcher verifies installer integrity and version; it
does not claim Windows trusts the publisher.

The private updater key stays outside the repository on the maintainer's PC. It
is not transmitted to GitHub. CI builds the unsigned packages and creates a
draft; the documented local finalization step verifies CI checksums, signs and
verifies the artifacts, adds the feed, and uploads only public signatures and
metadata. Drafts are not public download recommendations. Manual QA remains
required before marking a recipe Verified or publishing a release.

Changes to the updater public key require an explicit migration plan; changing
it silently would break the installed trust chain. Keep a secure retained backup
of the private key. No target app binary is patched, copied or redistributed.
