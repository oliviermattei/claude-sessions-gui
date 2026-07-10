# Security Policy

## Supported versions

Claude Sessions is distributed as prebuilt desktop installers via GitHub
Releases. Only the **latest released version** receives security fixes. Please
upgrade before reporting an issue to confirm it still reproduces.

| Version | Supported          |
| ------- | ------------------ |
| latest  | :white_check_mark: |
| older   | :x:                |

## Security model

A few properties are load-bearing for this app's safety — please keep them in
mind when auditing or contributing:

- **No silent mutation of transcripts.** Browsing, search and sorting are
  strictly read-only. Writes happen only as a direct result of an explicit user
  action: rename and set-color _append_ Claude-native rows (never rewriting or
  removing existing content); move rewrites the working directory into a copy and
  removes the original; delete sends files to the OS trash (recoverable). The app
  must never modify transcript content in place, and never write to, move, or
  delete a file the user did not act on. A regression that breaks this is a
  security bug.
- **Local-only.** The app performs no network requests to transmit session
  contents. Session data never leaves the user's machine.
- **User-defined launchers execute commands.** "Resume" launchers run
  user-configured command templates via the OS shell. Templates are under the
  user's control; a bug that lets untrusted transcript content influence the
  executed command (e.g. via `{cwd}`/`{id}` substitution) would be a serious
  vulnerability.

## Reporting a vulnerability

**Please do not open a public issue for security vulnerabilities.**

Report privately through one of:

1. **GitHub Security Advisories** — preferred. Use
   [*Report a vulnerability*](https://github.com/oliviermattei/claude-sessions-gui/security/advisories/new)
   on the repository's Security tab.
2. **Email** — `hello@feedvox.co`.

Please include:

- A description of the vulnerability and its impact.
- Steps to reproduce (a proof of concept if possible).
- Affected version and platform (OS + app version).

### What to expect

- Acknowledgement within **72 hours**.
- An initial assessment and severity classification.
- Coordinated disclosure: we will agree on a timeline and credit you in the
  release notes unless you prefer to stay anonymous.

Thank you for helping keep Claude Sessions and its users safe.
