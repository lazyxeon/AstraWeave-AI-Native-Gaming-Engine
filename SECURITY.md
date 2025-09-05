# Security Policy

## Supported Versions

Use this section to tell people about which versions of your project are currently being supported with security updates.

| Version | Supported          |
| ------- | ------------------ |
| 0.4.x   | :white_check_mark: |
| < 0.4.0 | :x:                |

## Reporting a Vulnerability

We take the security of Veilweaver: Threads of Eternity seriously. If you believe you've found a security vulnerability, please follow these steps:

1. **Do not disclose the vulnerability publicly**
2. **Email us at [security@example.com]** with:
   - A description of the vulnerability
   - Steps to reproduce the issue
   - Potential impact of the vulnerability
   - Any suggestions for remediation if you have them

## What to expect

- We will acknowledge receipt of your vulnerability report within 48 hours
- We will provide an initial assessment of the report within 5 business days
- We aim to release a fix for verified vulnerabilities within 30 days
- We will keep you informed about our progress throughout the process
- After the vulnerability has been addressed, we will publicly acknowledge your responsible disclosure (unless you prefer to remain anonymous)

## Security Measures

Veilweaver: Threads of Eternity implements several security measures:

1. **Dependency Scanning**: We use Dependabot to scan for vulnerable dependencies
2. **Static Analysis**: We use CodeQL and Clippy to identify potential security issues
3. **Security Audits**: We regularly audit our codebase with cargo-audit
4. **License Compliance**: We use cargo-deny to ensure all dependencies comply with our licensing requirements
5. **OpenSSF Scorecard**: We monitor our project's security posture using the OpenSSF Scorecard

## Security Best Practices

When contributing to Veilweaver: Threads of Eternity, please follow these security best practices:

1. **Keep dependencies updated** to their latest secure versions
2. **Validate all user inputs** before processing
3. **Follow the principle of least privilege** when implementing new features
4. **Use safe Rust practices** and avoid unsafe code blocks when possible
5. **Run security checks locally** before submitting pull requests

Thank you for helping keep Veilweaver: Threads of Eternity and its users safe!