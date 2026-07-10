<!--
Thanks for contributing! Please fill out this template.
Keep PRs focused — smaller PRs get reviewed and merged faster.
-->

## Description

<!-- What does this PR do and why? -->

## Related issue

<!-- e.g. Closes #123. If there's no issue, briefly explain the motivation above. -->

## Type of change

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that changes existing behavior)
- [ ] Documentation
- [ ] Build / CI / chore

## Checklist

- [ ] `pnpm build` passes (vue-tsc typecheck + vite build).
- [ ] `cargo fmt --check` and `cargo clippy -- -D warnings` pass for `src-tauri`.
- [ ] I followed the [Conventional Commits](https://www.conventionalcommits.org/) format.
- [ ] I updated docs / `CHANGELOG.md` where relevant.
- [ ] My change keeps browsing read-only and doesn't silently mutate `~/.claude`
      transcripts (writes only from explicit user actions; no in-place edits).
