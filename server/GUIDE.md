# Sunaba MCP Tool Guide for AI Agents

## Overview

Sunaba is a Model Context Protocol (MCP) tool that enables AI agents to write and execute code in isolated container environments. This guide provides comprehensive instructions for effectively utilizing Sunaba's capabilities.

## Supported Languages

- **Rust**: Compiled language with strong type safety
- **Go**: Compiled language with excellent concurrency support
- **Bun/JavaScript**: Runtime for JavaScript/TypeScript with modern features
- **Python**: ❌ **CURRENTLY BROKEN** - Execution fails due to entrypoint issue

## Quick Start

### 1. Initialize Workspace
```
sunaba_workspace_init({"languages": ["rust", "go", "bun"]})
```

### 2. Write Code Files
```
sunaba_file_write({
  "language": "rust",
  "rel_path": "main.rs",
  "content": "fn main() { println!('Hello World!'); }"
})
```

### 3. Execute Code
```
sunaba_exec_code({
  "language": "rust",
  "entrypoint": "main.rs"
})
```

## Core Functions

### Workspace Management

#### `sunaba_workspace_init`
Initializes workspace directories for specified languages.

**Parameters:**
- `languages`: Array of language strings ("python", "rust", "go", "bun")

**Example:**
```
sunaba_workspace_init({"languages": ["rust", "go", "bun"]})
```

**Response:**
```json
{
  "created": ["workspace/rust", "workspace/go", "workspace/bun"],
  "existing": []
}
```

### File Operations

#### `sunaba_file_write`
Writes files to the language-specific workspace.

**Parameters:**
- `language`: Target language
- `rel_path`: Relative path from workspace root
- `content`: File content as string
- `create_parents`: Optional, create parent directories

**Example:**
```
sunaba_file_write({
  "language": "go",
  "rel_path": "utils/helper.go",
  "content": "package utils\n\nfunc Helper() string { return 'help' }"
})
```

**Response:**
```json
{
  "bytes": 65,
  "created": true,
  "path": "workspace/go/utils/helper.go"
}
```

### Code Execution

#### `sunaba_exec_code`
Executes code in the target language environment.

**Parameters:**
- `language`: Target language
- `entrypoint`: Path to file to execute (relative to workspace)
- `code`: Inline code to execute (alternative to entrypoint)
- `args`: Command-line arguments array
- `compile_args`: Compilation arguments (Rust-specific)
- `timeout_secs`: Execution timeout in seconds
- `env`: Environment variables object

**Important Notes:**
- Use either `entrypoint` OR `code`, not both
- For Rust: compilation is automatic, binary artifact is generated
- For Go/Bun: direct execution without separate compilation
- Python: ❌ Currently broken - avoid using

**Examples:**

**File-based execution:**
```
sunaba_exec_code({
  "language": "rust",
  "entrypoint": "main.rs"
})
```

**Inline code execution:**
```
sunaba_exec_code({
  "language": "bun",
  "code": "console.log('Hello from Bun!')"
})
```

**With timeout:**
```
sunaba_exec_code({
  "language": "go",
  "code": "package main\nimport 'time'\nfunc main() { time.Sleep(10*time.Second) }",
  "timeout_secs": 3
})
```

**Response Format:**
```json
{
  "artifact": "/workspace/main_bin",  // Rust binary path
  "duration_ms": 450,                  // Execution time
  "exit_code": 0,                      // Process exit code
  "stderr": "",                        // Error output
  "stdout": "Hello World!\n"           // Standard output
}
```

### Environment Management

#### `sunaba_env_status`
Checks container status for a language.

**Example:**
```
sunaba_env_status({"language": "rust"})
```

**Response:**
```json
{
  "container_name": "rust_env",
  "exists": true,
  "pid": 123456,
  "running": true
}
```

#### `sunaba_env_start` / `sunaba_env_stop`
Manages container lifecycle.

**Examples:**
```
sunaba_env_stop({"language": "rust"})
sunaba_env_start({"language": "rust"})
```

## Language-Specific Guidelines

### Rust

**Strengths:**
- Strong type safety and performance
- Automatic compilation with artifact generation
- Excellent error messages

**Best Practices:**
- Use `entrypoint` for file-based execution
- Complex code works well (collections, iterators, etc.)
- Compilation time included in execution duration

**Example:**
```rust
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("key", "value");
    println!("Map: {:?}", map);
}
```

### Go

**Strengths:**
- Fast execution without compilation step
- Good for concurrent programming
- Simple module system

**Best Practices:**
- Use `package main` for executable programs
- Simple module imports work, complex ones may need setup
- Fast execution makes it ideal for quick tests

**Example:**
```go
package main

import "fmt"

func main() {
    numbers := []int{1, 2, 3, 4, 5}
    sum := 0
    for _, num := range numbers {
        sum += num
    }
    fmt.Printf("Sum: %d\n", sum)
}
```

### Bun/JavaScript

**Strengths:**
- Fastest execution time
- Full ES6 module support
- Modern JavaScript features
- Package.json support

**Best Practices:**
- Use ES6 modules for code organization
- Leverage modern JavaScript features
- Ideal for quick prototyping and testing

**Example with Modules:**
```javascript
// math.js
export function add(a, b) {
    return a + b;
}

// main.js
import { add } from './math.js';
console.log(`2 + 3 = ${add(2, 3)}`);
```

### Python ❌ (Currently Broken)

**Status:** Execution fails due to entrypoint configuration issue
**Workaround:** None available at this time
**Recommendation:** Avoid using Python until fix is implemented

## Error Handling

### Common Error Patterns

**File Not Found:**
```json
{
  "exit_code": 1,
  "stderr": "error: couldn't read `/workspace/nonexistent.rs`: No such file or directory",
  "stdout": ""
}
```

**Compilation Errors (Rust):**
```json
{
  "exit_code": 1,
  "stderr": "error[E0308]: mismatched types\n --> /workspace/main.rs:2:18",
  "stdout": ""
}
```

**Runtime Errors (Bun):**
```json
{
  "exit_code": 1,
  "stderr": "ReferenceError: undefinedVariable is not defined",
  "stdout": ""
}
```

**Timeout:**
```
"timeout"
```

### Best Practices for Error Handling

1. **Check exit_code**: Always verify `exit_code === 0` for success
2. **Read stderr**: Error messages provide detailed information
3. **Use timeouts**: Prevent infinite loops with `timeout_secs`
4. **Validate file paths**: Ensure files exist before execution

## Performance Optimization

### Execution Times (Approximate)

| Operation | Rust | Go | Bun |
|-----------|------|----|-----|
| Simple Hello World | ~400ms | ~300ms | ~80ms |
| Complex Program | ~1300ms | ~300ms | ~80ms |
| Module Import | N/A | ~13s (setup) | ~80ms |

### Optimization Tips

1. **Use Bun for quick tests**: Fastest execution time
2. **Use Rust for complex logic**: Best performance for CPU-intensive tasks
3. **Use Go for concurrent operations**: Built-in concurrency support
4. **Avoid unnecessary file writes**: Reuse existing files when possible

## Security Considerations

### Container Isolation
- Each language runs in separate Docker containers
- No cross-language interference
- Process isolation prevents security issues

### Resource Management
- Timeout functionality prevents resource exhaustion
- Clean process termination
- Memory and CPU constraints properly managed

## Troubleshooting

### Common Issues

**Python Not Working:**
- **Issue**: Python execution fails with "can't open file '/workspace/main.py'"
- **Cause**: Known bug in Python container configuration
- **Solution**: Avoid using Python until fix is implemented

**Go Module Import Issues:**
- **Issue**: "package not in std" errors
- **Cause**: Complex module imports require proper setup
- **Solution**: Use simple imports or proper go.mod configuration

**File Path Issues:**
- **Issue**: File not found errors
- **Cause**: Incorrect relative path specification
- **Solution**: Use correct relative paths from workspace root

### Debugging Steps

1. **Check environment status**: `sunaba_env_status`
2. **Verify file existence**: Use file writing response to confirm
3. **Test with simple code**: Start with "Hello World" examples
4. **Check error messages**: Read `stderr` for detailed information
5. **Use timeouts**: Prevent hanging processes

## Advanced Usage

### Multi-file Projects

**Rust Example:**
```rust
// lib.rs
pub fn helper() -> String {
    "help".to_string()
}

// main.rs
mod lib;

fn main() {
    println!("{}", lib::helper());
}
```

**Bun Example with Package.json:**
```json
{
  "name": "my-app",
  "type": "module",
  "dependencies": {}
}
```

### Environment Variables

```
sunaba_exec_code({
  "language": "bun",
  "code": "console.log(process.env.MY_VAR)",
  "env": {"MY_VAR": "test_value"}
})
```

### Command Line Arguments

```
sunaba_exec_code({
  "language": "rust",
  "entrypoint": "main.rs",
  "args": ["arg1", "arg2"]
})
```

## Best Practices Summary

1. **Initialize workspace first**: Always call `sunaba_workspace_init`
2. **Use appropriate language**: Choose based on task requirements
3. **Handle errors properly**: Check `exit_code` and read `stderr`
4. **Use timeouts**: Prevent infinite execution
5. **Test incrementally**: Start simple, then add complexity
6. **Avoid Python**: Until the execution issue is resolved
7. **Leverage modules**: For better code organization
8. **Monitor performance**: Use execution times to optimize

## Support and Updates

For issues, feature requests, or updates:
- Check the AUDIT.md for current status and known issues
- Monitor for Python execution fix
- Consider contributing to the project

---

*This guide will be updated as Sunaba evolves and new features are added.*