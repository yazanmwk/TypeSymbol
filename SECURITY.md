# Security Policy

## Supported Versions

Only the latest tagged release is supported for security fixes.

## Reporting a Vulnerability

If you discover a security issue, do not open a public issue first.

- Contact: open a private security advisory in this repository, or email the maintainer directly.
- Include:
  - impact summary
  - reproduction steps
  - affected version/tag
  - suggested mitigation (if known)

We will acknowledge reports as quickly as possible and coordinate a responsible disclosure timeline.

## Secret Handling Rules

- Never commit secrets, tokens, API keys, private keys, or `.env` files.
- Use GitHub Actions Secrets for CI credentials (example: `HOMEBREW_TAP_TOKEN`).
- Local release artifacts (`typesymbol-v*.tar.gz`, `checksums.txt`, `dist/`) must stay untracked.

## If a Secret Was Exposed

1. Revoke/rotate the credential immediately.
2. Remove it from repository history if needed.
3. Add/verify `.gitignore` protections.
4. Document remediation in the incident note or advisory.
