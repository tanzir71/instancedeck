# InstanceDeck portable package

Extract the entire archive to a folder you own, then run `instancedeck.exe`.
`ideck.exe` is the optional command-line launcher. Install the official target
apps separately; their binaries are not included.

Requires Windows 10 22H2 or Windows 11 and the Microsoft Edge WebView2 Runtime
for the selected x64/ARM64 architecture. This package installs no runtime and
makes no setup-time network requests. Obtain a missing WebView2 Runtime from
[Microsoft](https://developer.microsoft.com/microsoft-edge/webview2/).

Portable means no installer is required. Manager state still defaults to
`%LOCALAPPDATA%\InstanceDeck`; profiles use the configured data root. Set
`INSTANCEDECK_STATE_DIR` to an absolute directory before launching if you need a
separate manager state location. Keep `portable-mode.txt` beside the program.

To update, close InstanceDeck, extract the new portable archive to a new folder,
and launch that copy. Profile data remains in its configured location. The
portable build does not run the NSIS in-app installer.

Windows binaries are unsigned. Verify SHA-256 checksums and source provenance.
Do not disable Windows protection or install a root certificate to bypass a
warning. Profiles are not a security sandbox. Real account, pinning, installer,
Windows 10 and hardware verification remain manual checks; recipes are
Community.
