# Security policy

Report vulnerabilities through
[GitHub private vulnerability reporting](https://github.com/tanzir71/instancedeck/security/advisories/new).
Do not put credentials, authentication files, cookies, session transcripts or
unredacted account screenshots in a public issue. If private reporting is
unavailable, use the contact route at [tanziro.com](https://tanziro.com) to ask
for a private channel before sending sensitive details.

Include the InstanceDeck version, Windows build/architecture, affected recipe
and target app version, a minimal reproduction using synthetic data, and the
expected versus observed behavior. A redacted InstanceDeck diagnostic report is
useful. Never attach a target app's profile folder.

The project is in early development. No response-time guarantee or third-party
security audit is claimed. Only the newest published release receives fixes;
draft builds are for verification. Recipes stay Community until the documented
manual checks have evidence.

Read the [threat model](security/threat-model.md), [privacy policy](PRIVACY.md)
and [unsigned release policy](CODE_SIGNING_POLICY.md). Profiles separate app
state; they are not a security sandbox.
