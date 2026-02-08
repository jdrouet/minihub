# Security policy

## Reporting a vulnerability

If you discover a security vulnerability in minihub, please report it responsibly.

**Do not open a public issue.** Instead, email the maintainers directly or use GitHub's private vulnerability reporting feature.

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will acknowledge receipt within 48 hours and aim to provide a fix or mitigation within 7 days for critical issues.

## Scope

minihub is designed to run on a local network. It does not currently implement authentication or encryption. Do not expose it to the public internet.

Security-relevant areas:
- HTTP request handling (input validation, injection prevention)
- SQLite query construction (parameterised queries via sqlx)
- File path handling (no user-controlled file access)
- Configuration parsing (safe defaults)

## Supported versions

Only the latest version on `main` is supported with security fixes.
