# New package: Tanzir.InstanceDeck {{VERSION}}

Per-user unsigned NSIS installers for x64 and ARM64, built from the public
InstanceDeck repository. No target app binaries are bundled. WebView2 and the
official target apps are prerequisites. Uninstall preserves profile data.

Before creating an upstream PR:

- [ ] Publish the manually verified release; draft asset links are not public.
- [ ] Replace all template values with actual version and SHA-256 values.
- [ ] Validate against the current winget-pkgs schema and contribution policies.
- [ ] Run winget manifest validation and real install/uninstall tests on both
      architectures.
- [ ] Confirm unsigned installer acceptance and describe SmartScreen behavior
      accurately.
- [ ] Include the exact public source tag, build provenance and test evidence.

This is a preparation template. No upstream PR has been opened or communication
sent. Current schema 1.28.0 is present in microsoft/winget-pkgs; re-check at
submission.
