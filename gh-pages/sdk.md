---
layout: default
title: SDK (C ABI)
---

# C ABI SDK (astraweave-sdk)

AstraWeave exposes a stable C ABI for embedding the engine in non-Rust applications (C, C++, C#, Python, etc.).

## Features

- **cbindgen-generated header**: Automatic `.h` generation from Rust source
- **Opaque handles**: Engine objects represented as typed integer handles
- **Error codes**: Typed return values instead of exceptions
- **Miri-validated**: 17 tests confirming FFI safety (zero UB)

## API Surface

### Lifecycle

```c
#include "astraweave_sdk.h"

// Create engine instance
AwEngine* engine = aw_engine_create();

// Run one tick
AwError err = aw_engine_tick(engine, delta_time);

// Destroy
aw_engine_destroy(engine);
```

### Error Codes

| Code | Meaning |
|------|---------|
| `AW_OK` | Success |
| `AW_ERR_NULL_PTR` | Null pointer argument |
| `AW_ERR_INVALID_HANDLE` | Expired or invalid handle |
| `AW_ERR_INTERNAL` | Internal engine error |

### Entity Management

```c
// Spawn entity
AwEntityId entity = aw_entity_spawn(engine);

// Add component (raw bytes)
aw_entity_add_component(engine, entity, component_type_id, data, data_len);

// Query
const void* component = aw_entity_get_component(engine, entity, component_type_id);
```

## Safety Guarantees

- All exported functions check for null pointers before dereferencing
- Handle validation prevents use-after-free
- No panics cross the FFI boundary (all caught and converted to error codes)
- Thread safety documented per function

## Miri Validation

17 tests validated under Miri:
- Raw pointer construction and dereferencing
- C ABI calling convention correctness
- Handle lifecycle (create → use → destroy)
- Error path coverage (null, invalid, internal)

## Building the Header

```bash
cargo build -p astraweave-sdk
# Header generated at: target/astraweave_sdk.h
```

[← Back to Home](index.html) · [Architecture](architecture.html) · [Crate Index](crates.html)
