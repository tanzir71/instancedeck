# Recipe evidence

All eight built-in recipes remain Community. Generic app smoke testing was
deferred by the user; none is represented as locally account-verified.

Sources checked 2026-11-14:

- [VS Code command line](https://code.visualstudio.com/docs/configure/command-line):
  data and extensions directory flags.
  [Portable mode](https://code.visualstudio.com/docs/setup/portable) can
  override both flags.
- [Cursor's hosted CLI discussion](https://forum.cursor.com/t/add-code-options-to-cursor/48219):
  reported data/extensions flags; this is a community report, not an isolation
  guarantee. The recipe reports partial isolation because external state can
  remain shared.
- [Chromium user data directory](https://github.com/chromium/chromium/blob/main/docs/user_data_dir.md):
  separate user data directories and the command-line override.
- [Edge UserDataDir policy](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-browser-policies/userdatadir):
  enterprise policy can override the command-line flag.
- [Brave command-line flags](https://support.brave.app/hc/en-us/articles/360044860011-How-Do-I-Use-Command-Line-Flags-in-Brave):
  the user data directory flag.
- [Telegram Desktop switches](https://github.com/telegramdesktop/tdesktop/wiki/Command-Line-Switches):
  the legacy paired `-workdir <directory>` flag. Do not add the obsolete `-many`
  switch. Exact paired-argument tracking is tested with Unicode paths.

Discovery uses registry uninstall records before standard known-path fallbacks.
No recipe copies an installed binary or reads sign-in files. Custom recipes are
local manager metadata, use `custom-` ids, and start Experimental/Unknown. They
are excluded from the public site.
