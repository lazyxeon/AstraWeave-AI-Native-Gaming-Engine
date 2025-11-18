# Contributing to AstraWeave

Thank you for considering contributing to AstraWeave! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## How Can I Contribute?

### Reporting Bugs

- **Ensure the bug was not already reported** by searching on GitHub under [Issues](https://github.com/lazyxeon/AstraWeave-AI-Native-Gaming-Engine/issues).
- If you're unable to find an open issue addressing the problem, [open a new one](https://github.com/lazyxeon/AstraWeave-AI-Native-Gaming-Engine/issues/new/choose) using the bug report template.
- **For security vulnerabilities**, please follow our [Security Policy](SECURITY.md) instead of opening a public issue.

### Suggesting Enhancements

- [Open a new issue](https://github.com/lazyxeon/AstraWeave-AI-Native-Gaming-Engine/issues/new/choose) using the feature request template.
- Clearly describe the enhancement, its benefits, and potential implementation approaches.

### Pull Requests

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Development Environment Setup

1. Install Rust (nightly toolchain recommended):
   ```bash
   rustup toolchain install nightly
   rustup default nightly
   ```

2. Install development tools:
   ```bash
   cargo install cargo-audit cargo-deny cargo-criterion cargo-llvm-cov
   ```

3. Clone the repository:
   ```bash
   git clone https://github.com/lazyxeon/AstraWeave-AI-Native-Gaming-Engine.git
   cd AstraWeave-AI-Native-Gaming-Engine
   ```

4. Build the project:
   ```bash
   cargo build
   ```

5. Run tests:
   ```bash
   cargo test
   ```

## Style Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Follow the [Bevy Style Guide](https://bevyengine.org/learn/contributing/code-of-conduct/) for ECS patterns
- Use `rustfmt` to format your code before committing
- Use `clippy` to catch common mistakes and improve your code
- Write documentation for public APIs using doc comments (`///`)
- Include tests for new functionality
- Prefer descriptive variable names over abbreviations

## Security Best Practices

When contributing code, please follow these security best practices:

### General Security Guidelines

1. **Keep dependencies updated** to their latest secure versions
2. **Validate all user inputs** before processing
3. **Follow the principle of least privilege** when implementing new features
4. **Use safe Rust practices** and avoid unsafe code blocks when possible
5. **Run security checks locally** before submitting pull requests

### Rust-Specific Security Guidelines

1. **Minimize use of `unsafe` code**
   - Only use `unsafe` when absolutely necessary
   - Document why the unsafe code is needed and why it's safe
   - Consider alternatives before using unsafe code

2. **Handle errors properly**
   - Use `Result` and `Option` types appropriately
   - Don't use `.unwrap()` or `.expect()` in production code paths
   - Implement proper error handling and propagation
   - Use `.expect()` only for initialization or development-time assertions with clear messages

3. **Prevent memory safety issues**
   - Avoid raw pointers when possible
   - Use Rust's ownership system correctly
   - Be careful with lifetime annotations

4. **Secure resource management**
   - Ensure resources are properly cleaned up
   - Use RAII patterns with `Drop` trait
   - Be cautious with manual resource management

5. **Secure concurrency**
   - Use Rust's thread safety mechanisms
   - Avoid data races with proper synchronization
   - Be careful with shared mutable state

### Before Submitting a Pull Request

Run these security checks locally:

```bash
# Update dependencies
cargo update

# Check for security vulnerabilities
cargo audit

# Check for license compliance
cargo deny check

# Run the static analyzer
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-features

# Check formatting
cargo fmt --check
```

## Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that don't affect code meaning (formatting, etc.)
- **refactor**: Code change that neither fixes a bug nor adds a feature
- **perf**: Performance improvements
- **test**: Adding or correcting tests
- **chore**: Changes to build process or auxiliary tools

Format:
```
type(scope): brief description

More detailed explanation if needed.

Fixes #123
```

Examples:
- `feat(ecs): add support for sparse component storage`
- `fix(render): resolve depth buffer precision issues`
- `docs(contributing): update security guidelines`

## Testing Guidelines

- Write unit tests for individual functions and modules
- Write integration tests for cross-module interactions
- Use golden tests for deterministic output validation
- Run benchmarks for performance-critical code
- Aim for >80% code coverage on new features
- Include both positive and negative test cases

## Documentation Standards

- Document all public APIs with `///` doc comments
- Include examples in documentation where appropriate
- Update relevant markdown files in `/docs` for architectural changes
- Keep README.md up to date with significant changes
- Add entries to CHANGELOG.md following Keep a Changelog format

## Additional Resources

- [Rust Documentation](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Bevy Engine Documentation](https://bevyengine.org/learn/)
- [Rust Security](https://www.rust-lang.org/security.html)
- [Project Documentation](docs/)
- [Benchmarking Guide](docs/BENCHMARKING_GUIDE.md)

## Getting Help

- Join discussions in GitHub Issues
- Review existing documentation in `/docs`
- Check example projects in `/examples`
- Read the [Quick Start Guide](docs/QUICKSTART.md)

Thank you for contributing to AstraWeave!
