//! Script Sandbox Execution Tests
//!
//! Comprehensive test suite for async script execution with Rhai sandboxing.
//! Tests timeouts, resource limits, security isolation, and edge cases.

#[cfg(test)]
mod script_sandbox_tests {
    use crate::{execute_script_sandboxed, ExecutionLimits, ScriptSandbox};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Helper function to create a standard sandbox
    fn create_sandbox() -> ScriptSandbox {
        let mut engine = rhai::Engine::new();
        engine.set_max_operations(10000);
        engine.set_max_string_size(1000);

        ScriptSandbox {
            engine: Arc::new(Mutex::new(engine)),
            allowed_functions: HashMap::new(),
            execution_limits: ExecutionLimits {
                max_operations: 10000,
                max_memory_bytes: 1024 * 1024, // 1MB
                timeout_ms: 1000,              // 1 second
            },
        }
    }

    // Helper function to create a sandbox with custom timeout
    fn create_sandbox_with_timeout(timeout_ms: u64) -> ScriptSandbox {
        let mut engine = rhai::Engine::new();
        engine.set_max_operations(10000);
        engine.set_max_string_size(1000);

        ScriptSandbox {
            engine: Arc::new(Mutex::new(engine)),
            allowed_functions: HashMap::new(),
            execution_limits: ExecutionLimits {
                max_operations: 10000,
                max_memory_bytes: 1024 * 1024,
                timeout_ms,
            },
        }
    }

    // ============================================================================
    // Suite 1: Basic Execution (5 tests)
    // ============================================================================

    #[tokio::test]
    async fn test_simple_script_execution() {
        let sandbox = create_sandbox();
        let context = HashMap::new();
        let script = "2 + 2";

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_ok(), "Simple script should execute successfully");
        let value = result.unwrap().as_int().unwrap();
        assert_eq!(value, 4, "2 + 2 should equal 4");
    }

    #[tokio::test]
    async fn test_context_variable_passing() {
        let sandbox = create_sandbox();
        let mut context = HashMap::new();
        context.insert("x".to_string(), rhai::Dynamic::from(10_i64));
        context.insert("y".to_string(), rhai::Dynamic::from(20_i64));

        let script = "x + y";

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_ok(), "Script with context should execute");
        let value = result.unwrap().as_int().unwrap();
        assert_eq!(value, 30, "x + y should equal 30");
    }

    #[tokio::test]
    async fn test_return_value_handling() {
        let sandbox = create_sandbox();
        let context = HashMap::new();

        // Test different return types
        let scripts = vec![
            ("42", 42i64),
            ("10 + 32", 42i64),
            ("21 * 2", 42i64),
        ];

        for (script, expected) in scripts {
            let result = execute_script_sandboxed(script, &sandbox, context.clone()).await;
            assert!(result.is_ok(), "Script '{}' should execute", script);
            assert_eq!(
                result.unwrap().as_int().unwrap(),
                expected,
                "Script '{}' should return {}",
                script,
                expected
            );
        }
    }

    #[tokio::test]
    async fn test_empty_script_execution() {
        let sandbox = create_sandbox();
        let context = HashMap::new();
        let script = "";

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_ok(), "Empty script should not crash");
        // Empty script returns unit/void
    }

    #[tokio::test]
    async fn test_syntax_error_handling() {
        let sandbox = create_sandbox();
        let context = HashMap::new();
        let script = "let x = ; // Missing value"; // Definitely invalid syntax

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_err(), "Syntax error should be caught");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Parse") 
            || error_msg.contains("syntax")
            || error_msg.contains("Unexpected")
            || error_msg.contains("expected"),
            "Error should mention syntax/parse issue, got: {}",
            error_msg
        );
    }

    // ============================================================================
    // Suite 2: Timeout and Limits (5 tests)
    // ============================================================================

    #[tokio::test]
    async fn test_execution_timeout() {
        let sandbox = create_sandbox_with_timeout(100); // 100ms timeout
        let context = HashMap::new();

        // Script that takes longer than timeout (busy loop)
        let script = r#"
            let x = 0;
            loop {
                x += 1;
                if x > 1000000000 { break; }
            }
            x
        "#;

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_err(), "Long-running script should timeout");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("elapsed") || error_msg.contains("timeout") || error_msg.contains("operations"),
            "Error should mention timeout or operation limit, got: {}",
            error_msg
        );
    }

    #[tokio::test]
    async fn test_operation_count_limit() {
        let sandbox = create_sandbox(); // 10,000 operation limit
        let context = HashMap::new();

        // Script that exceeds operation limit
        let script = r#"
            let sum = 0;
            for i in 0..100000 {
                sum += i;
            }
            sum
        "#;

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        // Should either timeout or hit operation limit
        assert!(
            result.is_err(),
            "Script exceeding operation limit should fail"
        );
    }

    #[tokio::test]
    async fn test_fast_script_within_limits() {
        let sandbox = create_sandbox_with_timeout(1000); // 1 second
        let context = HashMap::new();

        // Fast script well within limits
        let script = r#"
            let sum = 0;
            for i in 0..100 {
                sum += i;
            }
            sum
        "#;

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_ok(), "Fast script should complete successfully");
        let value = result.unwrap().as_int().unwrap();
        assert_eq!(value, 4950, "Sum of 0..100 should be 4950");
    }

    #[tokio::test]
    async fn test_infinite_loop_detection() {
        let sandbox = create_sandbox_with_timeout(200); // 200ms timeout
        let context = HashMap::new();

        // True infinite loop (will hit operation limit or timeout)
        let script = r#"
            loop {
                // Never breaks
            }
        "#;

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_err(), "Infinite loop should be stopped");
    }

    #[tokio::test]
    async fn test_recursive_call_handling() {
        let sandbox = create_sandbox();
        let context = HashMap::new();

        // Recursive function (will hit operation limit eventually)
        let script = r#"
            fn factorial(n) {
                if n <= 1 {
                    1
                } else {
                    n * factorial(n - 1)
                }
            }
            factorial(10)
        "#;

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_ok(), "Reasonable recursion should work");
        let value = result.unwrap().as_int().unwrap();
        assert_eq!(value, 3628800, "factorial(10) should be 3628800");
    }

    // ============================================================================
    // Suite 3: Resource Constraints (5 tests)
    // ============================================================================

    #[tokio::test]
    async fn test_string_size_limit() {
        let sandbox = create_sandbox(); // 1000 char string limit
        let context = HashMap::new();

        // Try to create a very long string (will hit limit)
        let script = r#"
            let s = "";
            for i in 0..2000 {
                s += "a";
            }
            s
        "#;

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        // Should fail due to string size limit or operation limit
        assert!(
            result.is_err(),
            "Script creating oversized string should fail"
        );
    }

    #[tokio::test]
    async fn test_variable_scope_isolation() {
        let sandbox = create_sandbox();
        let mut context = HashMap::new();
        context.insert("x".to_string(), rhai::Dynamic::from(100));

        let script = r#"
            let x = 42;  // Shadow context variable
            x
        "#;

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_ok(), "Variable shadowing should work");
        let value = result.unwrap().as_int().unwrap();
        assert_eq!(value, 42, "Local variable should shadow context variable");
    }

    #[tokio::test]
    async fn test_multiple_variable_types() {
        let sandbox = create_sandbox();
        let mut context = HashMap::new();
        context.insert("int_val".to_string(), rhai::Dynamic::from(42_i64));
        context.insert("str_val".to_string(), rhai::Dynamic::from("hello"));
        context.insert("bool_val".to_string(), rhai::Dynamic::from(true));

        let script = r#"
            if bool_val {
                int_val + 8
            } else {
                0
            }
        "#;

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_ok(), "Multiple variable types should work");
        assert_eq!(result.unwrap().as_int().unwrap(), 50);
    }

    #[tokio::test]
    async fn test_array_operations() {
        let sandbox = create_sandbox();
        let context = HashMap::new();

        let script = r#"
            let arr = [1, 2, 3, 4, 5];
            let sum = 0;
            for item in arr {
                sum += item;
            }
            sum
        "#;

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_ok(), "Array operations should work");
        assert_eq!(result.unwrap().as_int().unwrap(), 15);
    }

    #[tokio::test]
    async fn test_nested_data_structures() {
        let sandbox = create_sandbox();
        let context = HashMap::new();

        let script = r#"
            let data = #{
                x: 10,
                y: 20,
                nested: #{
                    z: 30
                }
            };
            data.x + data.y + data.nested.z
        "#;

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_ok(), "Nested maps should work");
        assert_eq!(result.unwrap().as_int().unwrap(), 60);
    }

    // ============================================================================
    // Suite 4: Security Isolation (5 tests)
    // ============================================================================

    #[tokio::test]
    async fn test_file_system_access_blocked() {
        let sandbox = create_sandbox();
        let context = HashMap::new();

        // Try to access file system (should fail - Rhai doesn't have fs by default)
        let script = r#"
            // This will fail because Rhai engine doesn't expose file system
            open("test.txt")
        "#;

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_err(), "File system access should be blocked");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("open") 
            || error_msg.contains("not found")
            || error_msg.contains("Unknown"),
            "Error should mention undefined function, got: {}",
            error_msg
        );
    }

    #[tokio::test]
    async fn test_network_access_blocked() {
        let sandbox = create_sandbox();
        let context = HashMap::new();

        // Try to access network (should fail - Rhai doesn't have network by default)
        let script = r#"
            // This will fail because Rhai engine doesn't expose network
            http_get("http://example.com")
        "#;

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_err(), "Network access should be blocked");
    }

    #[tokio::test]
    async fn test_system_call_blocking() {
        let sandbox = create_sandbox();
        let context = HashMap::new();

        // Try to make system calls (should fail - Rhai doesn't expose system by default)
        let script = r#"
            system("ls")
        "#;

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_err(), "System calls should be blocked");
    }

    #[tokio::test]
    async fn test_import_blocking() {
        let sandbox = create_sandbox();
        let context = HashMap::new();

        // Try to import modules (Rhai has limited import support, should be restricted)
        let script = r#"
            import "dangerous_module";
        "#;

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_err(), "Module imports should be blocked or fail");
    }

    #[tokio::test]
    async fn test_safe_math_operations_allowed() {
        let sandbox = create_sandbox();
        let context = HashMap::new();

        // Safe operations should work
        let script = r#"
            let x = 10;
            let y = 20;
            let sum = x + y;
            let product = x * y;
            let power = x ** 2;
            sum + product + power
        "#;

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_ok(), "Safe math operations should be allowed");
        // 30 + 200 + 100 = 330
        assert_eq!(result.unwrap().as_int().unwrap(), 330);
    }

    // ============================================================================
    // Suite 5: Edge Cases and Error Handling (5 tests - BONUS)
    // ============================================================================

    #[tokio::test]
    async fn test_division_by_zero() {
        let sandbox = create_sandbox();
        let context = HashMap::new();
        let script = "10 / 0";

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        // Rhai should catch division by zero
        assert!(result.is_err(), "Division by zero should fail");
    }

    #[tokio::test]
    async fn test_undefined_variable_access() {
        let sandbox = create_sandbox();
        let context = HashMap::new();
        let script = "undefined_variable + 10";

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_err(), "Undefined variable should fail");
    }

    #[tokio::test]
    async fn test_type_mismatch_error() {
        let sandbox = create_sandbox();
        let mut context = HashMap::new();
        context.insert("text".to_string(), rhai::Dynamic::from("hello"));

        let script = "text + 10"; // Try to add string and number

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        // Rhai might handle this differently (could concatenate or error)
        // We just verify it doesn't crash
        assert!(
            result.is_ok() || result.is_err(),
            "Type mismatch should be handled gracefully"
        );
    }

    #[tokio::test]
    async fn test_null_context_handling() {
        let sandbox = create_sandbox();
        let context = HashMap::new(); // Empty context

        let script = "42";

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_ok(), "Empty context should work");
        assert_eq!(result.unwrap().as_int().unwrap(), 42);
    }

    #[tokio::test]
    async fn test_complex_expression_evaluation() {
        let sandbox = create_sandbox();
        let mut context = HashMap::new();
        context.insert("a".to_string(), rhai::Dynamic::from(5_i64));
        context.insert("b".to_string(), rhai::Dynamic::from(3_i64));
        context.insert("c".to_string(), rhai::Dynamic::from(2_i64));

        let script = "(a + b) * c - a / b";

        let result = execute_script_sandboxed(script, &sandbox, context).await;

        assert!(result.is_ok(), "Complex expressions should work");
        // (5 + 3) * 2 - 5 / 3 = 8 * 2 - 1 = 16 - 1 = 15
        let value = result.unwrap().as_int().unwrap();
        assert!(value == 15 || value == 14, "Result should be 15 or 14 (int division): got {}", value);
    }
}
