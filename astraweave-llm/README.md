# AstraWeave LLM Integration

The AstraWeave LLM integration provides secure, validated connections to Large Language Models for AI-driven game companion behavior. This module enables NPCs and companions to make intelligent decisions based on game state while maintaining strict security and validation constraints.

## Features

- **Multiple LLM Clients**: Support for MockLlm (testing), Ollama (local), and OpenAI-compatible APIs
- **Tool Validation**: Strict allowlist-based validation ensures LLMs can only use approved actions
- **JSON Schema Enforcement**: Structured output validation prevents malformed responses
- **Comprehensive Testing**: Unit and integration tests covering all scenarios
- **Error Handling**: Robust error handling for network failures, invalid JSON, and disallowed actions
- **Security**: No dynamic code execution, input sanitization, constraint enforcement

## Quick Start

### Basic Usage

```rust
use astraweave_llm::{MockLlm, plan_from_llm};
use astraweave_core::{WorldSnapshot, ToolRegistry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = MockLlm;
    let world_snapshot = create_world_snapshot();
    let tool_registry = create_tool_registry();
    
    let plan = plan_from_llm(&client, &world_snapshot, &tool_registry).await?;
    println!("Generated plan: {}", serde_json::to_string_pretty(&plan)?);
    
    Ok(())
}
```

### Running Examples

```bash
# Basic example with MockLlm
cargo run -p llm_toolcall

# Comprehensive integration example
cargo run -p llm_integration

# With Ollama (requires Ollama running locally)
OLLAMA_URL=http://localhost:11434 OLLAMA_MODEL=llama2 cargo run -p llm_integration

# With local OpenAI-compatible API
LOCAL_LLM_URL=http://localhost:5000 LOCAL_LLM_MODEL=gpt-3.5-turbo cargo run -p llm_integration
```

## LLM Clients

### MockLlm (Always Available)

The MockLlm client provides deterministic output for testing and development. No external dependencies required.

```rust
use astraweave_llm::MockLlm;

let client = MockLlm;
let response = client.complete("generate a plan").await?;
```

### Ollama Client (Feature: `ollama`)

Connects to local Ollama instances for privacy-focused AI inference.

```rust
use astraweave_llm::OllamaClient;

let client = OllamaClient {
    url: "http://localhost:11434".to_string(),
    model: "llama2".to_string(),
};
```

### Local HTTP Client (Feature: `ollama`)

OpenAI-compatible API client for local inference servers like text-generation-webui, LocalAI, etc.

```rust
use astraweave_llm::LocalHttpClient;

// Without API key (for local services)
let client = LocalHttpClient::new(
    "http://localhost:5000".to_string(),
    "local-model".to_string(),
);

// With API key (for services that require authentication)
let client = LocalHttpClient::with_api_key(
    "https://api.openai.com".to_string(),
    "gpt-3.5-turbo".to_string(),
    "your-api-key".to_string(),
);
```

## Tool Validation System

The LLM integration includes a comprehensive validation system that operates at multiple levels:

1. **Schema Validation**: Ensures JSON structure matches expected format
2. **Tool Registry Validation**: Verifies all actions are in the allowed tool set  
3. **Engine Validation**: Runtime checks for cooldowns, line-of-sight, and other constraints

### Defining Tools

```rust
use astraweave_core::{ToolRegistry, ToolSpec, Constraints};

let registry = ToolRegistry {
    tools: vec![
        ToolSpec {
            name: "move_to".into(),
            args: [("x", "i32"), ("y", "i32")]
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        },
        ToolSpec {
            name: "throw".into(),
            args: [("item", "enum[smoke,grenade]"), ("x", "i32"), ("y", "i32")]
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        },
    ],
    constraints: Constraints {
        enforce_cooldowns: true,
        enforce_los: true,
        enforce_stamina: true,
    },
};
```

### Supported Actions

- **MoveTo**: Move companion to specified coordinates
- **Throw**: Throw items (smoke, grenade, flashbang) at target location
- **CoverFire**: Provide covering fire at target for specified duration  
- **Revive**: Revive allied units

## Testing

The module includes comprehensive test coverage:

```bash
# Run all tests
cargo test -p astraweave-llm

# Run specific test categories
cargo test -p astraweave-llm --lib          # Unit tests
cargo test -p astraweave-llm --test integration_test  # Integration tests
```

## Security Features

- **Allowlist-only Actions**: Only pre-approved actions can be executed
- **No Dynamic Code**: All actions are statically defined, no script execution
- **Input Sanitization**: JSON parsing prevents injection attacks
- **Validation Pipeline**: Multi-layer validation catches malformed or malicious input
- **Timeout Protection**: Network requests have configurable timeouts
- **Error Containment**: Failures are isolated and don't crash the game engine

## Configuration

### Environment Variables

- `OLLAMA_URL`: Ollama server URL (default: http://localhost:11434)
- `OLLAMA_MODEL`: Ollama model name (default: llama2)
- `LOCAL_LLM_URL`: Local LLM server URL
- `LOCAL_LLM_MODEL`: Local LLM model name
- `LOCAL_LLM_API_KEY`: API key for authenticated services

### Cargo Features

- `ollama`: Enables Ollama and LocalHttpClient support (requires reqwest)

## Error Handling

The LLM integration provides detailed error messages for common failure modes:

- **Network Errors**: Connection timeouts, server unavailable
- **JSON Parsing**: Malformed JSON responses from LLM
- **Validation Errors**: Disallowed tools, missing fields, type mismatches
- **API Errors**: HTTP error codes, authentication failures

## Integration with Game Engine

The LLM module integrates seamlessly with the AstraWeave game engine:

1. **World Snapshot**: Current game state is serialized to JSON
2. **Tool Registry**: Available actions are defined by game rules
3. **Plan Generation**: LLM generates action sequence based on game state
4. **Validation**: Plan is validated against tool registry and constraints
5. **Execution**: Validated actions are executed by the game engine

## Performance Considerations

- **Async Operations**: All LLM calls are non-blocking
- **Caching**: Consider implementing response caching for repeated scenarios
- **Timeouts**: Network timeouts prevent hanging operations
- **Local Inference**: Ollama and local APIs eliminate network latency
- **Batch Processing**: Multiple requests can be processed concurrently

## Extending the System

### Adding New LLM Clients

Implement the `LlmClient` trait:

```rust
use astraweave_llm::LlmClient;
use anyhow::Result;

struct MyLlmClient;

#[async_trait::async_trait]
impl LlmClient for MyLlmClient {
    async fn complete(&self, prompt: &str) -> Result<String> {
        // Your implementation here
        todo!()
    }
}
```

### Adding New Actions

1. Add action variant to `ActionStep` enum in astraweave-core
2. Update validation logic in `parse_llm_plan` function
3. Add tool specification to registry
4. Update JSON schema in prompt generation

## Documentation

- [Tool Validation System](../docs/src/architecture/tool-validation.md)
- [Build Quick Reference](../docs/BUILD_QUICK_REFERENCE.md)

## License

This module is part of the AstraWeave AI Gaming Engine and follows the same licensing terms.