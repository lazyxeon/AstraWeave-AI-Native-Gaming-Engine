# Security Policy

## Supported Versions

Only the latest `main` branch is actively supported with security updates. Published releases inherit fixes when tagged.

## Reporting a Vulnerability

- Email: **security@astraweave.dev**
- PGP: Available upon request via the email above

Please include:
- A detailed description of the issue
- Steps to reproduce
- Potential impact assessment
- Any suggested mitigations

We request a 72-hour window before public disclosure to coordinate fixes and releases.

## Responsible Disclosure

- Do not create GitHub issues for security reports.
- Avoid sharing proof-of-concept code publicly before we address the vulnerability.
- We will acknowledge receipt within 48 hours and provide status updates until resolution.

## Security Update Process

1. Confirm and reproduce the report in a secure environment.
2. Prioritize remediation based on severity using CVSS.
3. Develop patches and add regression tests when applicable.
4. Coordinate release notes and advisories in `CHANGELOG.md`.
5. Notify the reporter once the fix is released.

## Scope

This policy covers all crates under the AstraWeave workspace, tooling in `tools/`, and official Docker/installer artifacts.

Thank you for helping keep the AstraWeave community secure.