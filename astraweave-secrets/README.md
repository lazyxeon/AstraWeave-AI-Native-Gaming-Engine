# astraweave-secrets

Secure secret management for AstraWeave using OS keychains.

## Usage

### CLI Tool

```bash
# Interactive setup
cargo run --bin aw_secrets -- init

# Store a secret
cargo run --bin aw_secrets -- set llm.api_key

# Retrieve a secret
cargo run --bin aw_secrets -- get llm.api_key

# Delete a secret
cargo run --bin aw_secrets -- delete llm.api_key
```

### Rust API

```rust
use astraweave_secrets::{SecretManager, SecretValue};

let manager = SecretManager::global();

// Store secret
manager.set("my.key", SecretValue::from_str("secret_value"))?;

// Retrieve secret
let value = manager.get("my.key")?;
println!("{}", value.as_str()?);
```

## Storage

Secrets are stored in:
- **Windows**: Credential Manager
- **macOS**: Keychain
- **Linux**: Secret Service (via keyring crate)
