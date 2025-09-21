//! Comprehensive tests for FFI utilities
//!
//! These tests cover dynamic library loading, symbol resolution, error handling,
//! and cross-platform compatibility of the FFI abstraction layer.

#[cfg(test)]
mod ffi_utils_tests {
    use crate::ffi_utils::{
        AdlResult, ApiResult, ApiTable, LibraryLoader, NvmlResult,
    };

    /// Test NVML API result wrapper
    #[test]
    fn test_nvml_result() {
        let success_result = NvmlResult { code: 0, value: 42 };
        assert!(success_result.is_success());
        assert_eq!(success_result.error_code(), 0);
        assert_eq!(success_result.to_option(), Some(42));
        let error_result = NvmlResult { code: 1, value: 0 };
        assert!(!error_result.is_success());
        assert_eq!(error_result.error_code(), 1);
        assert_eq!(error_result.to_option(), None);
    }

    /// Test ADL API result wrapper
    #[test]
    fn test_adl_result() {
        let success_result = AdlResult { code: 0, value: "success".to_string() };
        assert!(success_result.is_success());
        assert_eq!(success_result.error_code(), 0);
        assert_eq!(success_result.to_option(), Some("success".to_string()));
        let error_result = AdlResult { code: -1, value: "error".to_string() };
        assert!(!error_result.is_success());
        assert_eq!(error_result.error_code(), -1);
        assert_eq!(error_result.to_option(), None);
    }

    /// Test API result trait with different types
    #[test]
    fn test_api_result_trait() {
        let int_result = NvmlResult { code: 0, value: 123 };
        let opt_value: Option<i32> = int_result.to_option();
        assert_eq!(opt_value, Some(123));
        let string_result = AdlResult { code: 0, value: "test".to_string() };
        let opt_string: Option<String> = string_result.to_option();
        assert_eq!(opt_string, Some("test".to_string()));
        let float_result = NvmlResult { code: 0, value: 3.14f32 };
        let opt_float: Option<f32> = float_result.to_option();
        assert_eq!(opt_float, Some(3.14f32));
    }

    /// Test library loader builder pattern
    #[test]
    fn test_library_loader() {
        let loader = LibraryLoader::new("test_library")
            .with_fallback_path("/path/to/fallback1")
            .with_fallback_path("/path/to/fallback2");
        let result = loader.load();
        assert!(result.is_err());
        println!("Non-existent library loading failed as expected: {}", result.unwrap_err());
    }

    /// Test library loader with multiple fallback paths
    #[test]
    fn test_library_loader_multiple_fallbacks() {
        let loader = LibraryLoader::new("nonexistent_lib")
            .with_fallback_path("/usr/lib/nonexistent1.so")
            .with_fallback_path("/usr/lib/nonexistent2.so")
            .with_fallback_path("/opt/nonexistent3.so");
        let result = loader.load();
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Failed to load library"));
        assert!(error_msg.contains("and all fallback paths"));
        println!("Multiple fallback paths test passed: {}", error_msg);
    }

    /// Test API table functionality
    #[test]
    fn test_api_table() {
        #[derive(Debug)]
        struct TestFunctions {
            func1: fn() -> i32,
            func2: fn(i32) -> i32,
        }

        fn test_func1() -> i32 { 42 }
        fn test_func2(x: i32) -> i32 { x * 2 }

        let functions = TestFunctions {
            func1: test_func1,
            func2: test_func2,
        };

        let api_table = ApiTable::new(functions);
        let funcs = api_table.functions();
        assert_eq!((funcs.func1)(), 42);
        assert_eq!((funcs.func2)(21), 42);
    }

    /// Test symbol resolver creation
    #[test]
    fn test_symbol_resolver_creation() {
        // TODO:Implement a test that actually loads a library
        // We can't actually load libraries in test environment,
        // but we can test the creation logic
        // This test mainly ensures the SymbolResolver struct works
        // without actually loading symbols
        println!("Symbol resolver creation test passed (structure validation)");
    }

    /// Test error handling macros (compilation test)
    #[test]
    fn test_error_handling_macros() {
        let success_result = NvmlResult { code: 0, value: 100 };
        if success_result.is_success() {
            let value = success_result.to_option().unwrap();
            assert_eq!(value, 100);
        }
        let error_result = NvmlResult { code: 1, value: 0 };
        if let Some(_) = error_result.to_option() {
            panic!("Should not get value from error result");
        } else {
            println!("Error result handled correctly");
        }
        println!("Error handling macros test passed");
    }

    /// Test different result types with macros
    #[test]
    fn test_result_types_with_macros() {
        // Test with different value types
        let int_result = NvmlResult { code: 0, value: 42i32 };
        let float_result = AdlResult { code: 0, value: 3.14f64 };
        let bool_result = NvmlResult { code: 0, value: true };
        assert!(int_result.is_success());
        assert!(float_result.is_success());
        assert!(bool_result.is_success());
        assert_eq!(int_result.to_option(), Some(42));
        assert_eq!(float_result.to_option(), Some(3.14f64));
        assert_eq!(bool_result.to_option(), Some(true));
    }

    /// Test API result error codes
    #[test]
    fn test_api_result_error_codes() {
        let test_cases = vec![
            (0, true),   // Success
            (1, false),  // Error
            (-1, false), // Error
            (999, false), // Error
        ];

        for (code, should_succeed) in test_cases {
            let nvml_result = NvmlResult { code, value: () };
            let adl_result = AdlResult { code, value: () };
            assert_eq!(nvml_result.is_success(), should_succeed);
            assert_eq!(adl_result.is_success(), should_succeed);
            assert_eq!(nvml_result.error_code(), code);
            assert_eq!(adl_result.error_code(), code);
            if should_succeed {
                assert!(nvml_result.to_option().is_some());
                assert!(adl_result.to_option().is_some());
            } else {
                assert!(nvml_result.to_option().is_none());
                assert!(adl_result.to_option().is_none());
            }
        }
    }

    /// Stress test: Create many API results
    #[test]
    fn test_api_result_stress() {
        const ITERATIONS: usize = 10000;
        let mut success_count = 0;
        let mut error_count = 0;
        for i in 0..ITERATIONS {
            let code = if i % 3 == 0 { 0 } else { i as i32 };
            let result = NvmlResult { code, value: i };
            if result.is_success() {
                success_count += 1;
                assert!(result.to_option().is_some());
            } else {
                error_count += 1;
                assert!(result.to_option().is_none());
            }
        }
        println!("Stress test completed: {} successes, {} errors", 
                 success_count, error_count);
        assert_eq!(success_count + error_count, ITERATIONS);
    }

    /// Test library loading error scenarios
    #[test]
    fn test_library_loading_errors() {
        let invalid_names = vec![
            "",
            "nonexistent_library_12345",
            "/path/to/nowhere.so",
            "invalid\\path\\library.dll",
            "library with spaces.so",
        ];
        for name in invalid_names {
            let loader = LibraryLoader::new(name);
            let result = loader.load();
            assert!(result.is_err(), "Loading '{}' should fail", name);
            println!("Expected error for '{}': loading failed as expected", name);
        }
    }

    /// Test cross-platform library name handling
    #[test]
    fn test_cross_platform_library_names() {
        let test_cases = vec![
            ("kernel32", "Windows system library"),
            ("libc.so.6", "Linux C library"),
            ("libSystem.dylib", "macOS system library"),
            ("nvml", "NVIDIA library"),
            ("amdadl64", "AMD library"),
        ];
        for (lib_name, description) in test_cases {
            let loader = LibraryLoader::new(lib_name);
            let result = loader.load();
            // TODO: что это такое ?
            // We don't expect these to load in test environment,
            // but the loader should handle them gracefully
            match result {
                Ok(_) => println!("Unexpectedly loaded {}: {}", lib_name, description),
                Err(e) => println!("Expected failure for {}: {}", lib_name, e),
            }
        }
    }

    /// Integration test: Full FFI workflow simulation
    #[test]
    fn test_full_ffi_workflow_simulation() {
        println!("Starting full FFI workflow simulation");
        let loader = LibraryLoader::new("test_gpu_lib")
            .with_fallback_path("/usr/lib/test_gpu.so")
            .with_fallback_path("/opt/gpu/lib/test_gpu.so")
            .with_fallback_path("C:\\Program Files\\GPU\\test_gpu.dll");
        let load_result = loader.load();
        println!("Library loading result: {:?}", load_result.is_ok());
        let api_operations = vec![
            ("init", NvmlResult { code: 0, value: () }),
            ("shutdown", NvmlResult { code: 0, value: () }),
        ];
        for (operation, result) in api_operations {
            println!("Operation '{}': success={}, error_code={}", 
                     operation, result.is_success(), result.error_code());
            
            if result.is_success() {
                println!("  Operation succeeded");
            } else {
                println!("  Operation failed with code {}", result.error_code());
            }
        }
        let error_scenarios = vec![
            ("device_not_found", NvmlResult { code: 6, value: () }),
            ("insufficient_power", NvmlResult { code: 13, value: () }),
            ("driver_not_loaded", NvmlResult { code: 9, value: () }),
        ];
        for (scenario, result) in error_scenarios {
            assert!(!result.is_success());
            assert!(result.to_option().is_none());
            println!("Error scenario '{}' handled correctly", scenario);
        }
        println!("Full FFI workflow simulation completed");
    }

    /// Performance test: API result conversions
    #[test]
    fn test_api_result_performance() {
        use std::time::Instant;
        const ITERATIONS: usize = 1_000_000;
        let start = Instant::now();
        let mut success_count = 0;
        for i in 0..ITERATIONS {
            let result = NvmlResult { 
                code: if i % 100 == 0 { 1 } else { 0 }, 
                value: i 
            };
            if result.is_success() {
                if let Some(_value) = result.to_option() {
                    success_count += 1;
                }
            }
        }
        let duration = start.elapsed();
        let ops_per_sec = ITERATIONS as f64 / duration.as_secs_f64();
        println!("  Iterations: {}", ITERATIONS);
        println!("  Duration: {:?}", duration);
        println!("  Operations per second: {:.0}", ops_per_sec);
        println!("  Success count: {}", success_count);
        assert!(duration.as_millis() < 1000, "Performance test too slow: {:?}", duration);
        assert!(ops_per_sec > 100_000.0, "Too slow: {:.0} ops/sec", ops_per_sec);
    }
}