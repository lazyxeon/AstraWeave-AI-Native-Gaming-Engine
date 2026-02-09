# astraweave-sdk

C ABI surface for embedding AstraWeave in non-Rust applications.

## Overview

Exposes the engine through a stable C-compatible interface designed for `cbindgen` header generation. Supports C, C++, C#, Python, and other FFI-capable languages.

## C API Functions

| Function | Description |
|----------|-------------|
| `aw_world_create()` | Create a new physics world |
| `aw_world_destroy()` | Destroy a world handle |
| `aw_world_tick()` | Advance simulation by one step |
| `aw_world_snapshot_json()` | Export world snapshot as JSON |
| `aw_world_submit_intent_json()` | Submit an AI intent (JSON) |
| `aw_world_set_snapshot_callback()` | Register snapshot update callback |
| `aw_version()` / `aw_version_string()` | Query engine version |
| `aw_last_error_string()` | Retrieve last error message |

## Error Codes

| Code | Constant | Meaning |
|------|----------|---------|
| 0 | `AW_OK` | Success |
| 1 | `AW_ERR_NULL` | Null pointer |
| 2 | `AW_ERR_PARAM` | Invalid parameter |
| 3 | `AW_ERR_PARSE` | JSON parse failure |
| 4 | `AW_ERR_EXEC` | Execution error |

## Usage (C)

```c
#include "astraweave.h"

AWWorld* world = aw_world_create();
aw_world_tick(world);
const char* json = aw_world_snapshot_json(world);
aw_world_destroy(world);
```

## License

MIT
