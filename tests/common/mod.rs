/// Shared test utilities for all test modules
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use anyhow::Result;

/// Test fixture for creating temporary file structures
pub struct TestProject {
    pub temp_dir: TempDir,
    pub root_path: PathBuf,
}

impl TestProject {
    /// Create a new test project with a temporary directory
    pub fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let root_path = temp_dir.path().to_path_buf();
        Ok(Self { temp_dir, root_path })
    }

    /// Create a file with content in the test project
    pub fn create_file<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<PathBuf> {
        let file_path = self.root_path.join(path);
        
        // Create parent directories if they don't exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(&file_path, content)?;
        Ok(file_path)
    }
    
    /// Create a directory in the test project
    pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        let dir_path = self.root_path.join(path);
        fs::create_dir_all(&dir_path)?;
        Ok(dir_path)
    }
    
    /// Create a TypeScript file with specified content
    pub fn create_ts_file<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<PathBuf> {
        let ts_path = format!("{}.ts", path.as_ref().to_string_lossy());
        self.create_file(ts_path, content)
    }
    
    /// Create a JavaScript file with specified content
    pub fn create_js_file<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<PathBuf> {
        let js_path = format!("{}.js", path.as_ref().to_string_lossy());
        self.create_file(js_path, content)
    }
    
    /// Create a package.json file with dependencies
    pub fn create_package_json(&self, dependencies: &[(&str, &str)], dev_dependencies: &[(&str, &str)]) -> Result<PathBuf> {
        let mut deps = serde_json::Map::new();
        for (name, version) in dependencies {
            deps.insert(name.to_string(), serde_json::Value::String(version.to_string()));
        }
        
        let mut dev_deps = serde_json::Map::new();
        for (name, version) in dev_dependencies {
            dev_deps.insert(name.to_string(), serde_json::Value::String(version.to_string()));
        }
        
        let package_json = serde_json::json!({
            "name": "test-project",
            "version": "1.0.0",
            "dependencies": deps,
            "devDependencies": dev_deps
        });
        
        let content = serde_json::to_string_pretty(&package_json)?;
        self.create_file("package.json", &content)
    }
    
    /// Get the absolute path for a relative path within the project
    pub fn path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.root_path.join(path)
    }
}

/// Sample TypeScript files for testing
pub struct SampleFiles;

impl SampleFiles {
    /// A component with type issues
    pub fn component_with_type_issues() -> &'static str {
        r#"
import React from 'react';

interface Props {
    name: string;
    age?: any; // Type issue: using 'any'
}

const UserCard: React.FC<Props> = ({ name, age }) => {
    const handleClick = (event: any) => { // Another 'any' usage
        console.log(event);
    };
    
    return (
        <div onClick={handleClick}>
            <h1>{name}</h1>
            <p>Age: {age}</p>
        </div>
    );
};

export default UserCard;
"#
    }
    
    /// A large component (over 100 lines)
    pub fn large_component() -> &'static str {
        let mut content = String::from("import React from 'react';\n\n");
        content.push_str("const LargeComponent = () => {\n");
        
        // Add many lines to make it large
        for i in 1..=120 {
            content.push_str(&format!("  const var{} = 'value{}';\n", i, i));
        }
        
        content.push_str("  return <div>Large Component</div>;\n");
        content.push_str("};\n\n");
        content.push_str("export default LargeComponent;");
        
        // Return static string for testing (in real implementation, this would be dynamic)
        // For testing purposes, we'll use a simplified version
        r#"
import React from 'react';

const LargeComponent = () => {
  const var1 = 'value1';
  const var2 = 'value2';
  const var3 = 'value3';
  const var4 = 'value4';
  const var5 = 'value5';
  const var6 = 'value6';
  const var7 = 'value7';
  const var8 = 'value8';
  const var9 = 'value9';
  const var10 = 'value10';
  const var11 = 'value11';
  const var12 = 'value12';
  const var13 = 'value13';
  const var14 = 'value14';
  const var15 = 'value15';
  const var16 = 'value16';
  const var17 = 'value17';
  const var18 = 'value18';
  const var19 = 'value19';
  const var20 = 'value20';
  const var21 = 'value21';
  const var22 = 'value22';
  const var23 = 'value23';
  const var24 = 'value24';
  const var25 = 'value25';
  const var26 = 'value26';
  const var27 = 'value27';
  const var28 = 'value28';
  const var29 = 'value29';
  const var30 = 'value30';
  const var31 = 'value31';
  const var32 = 'value32';
  const var33 = 'value33';
  const var34 = 'value34';
  const var35 = 'value35';
  const var36 = 'value36';
  const var37 = 'value37';
  const var38 = 'value38';
  const var39 = 'value39';
  const var40 = 'value40';
  const var41 = 'value41';
  const var42 = 'value42';
  const var43 = 'value43';
  const var44 = 'value44';
  const var45 = 'value45';
  const var46 = 'value46';
  const var47 = 'value47';
  const var48 = 'value48';
  const var49 = 'value49';
  const var50 = 'value50';
  const var51 = 'value51';
  const var52 = 'value52';
  const var53 = 'value53';
  const var54 = 'value54';
  const var55 = 'value55';
  const var56 = 'value56';
  const var57 = 'value57';
  const var58 = 'value58';
  const var59 = 'value59';
  const var60 = 'value60';
  const var61 = 'value61';
  const var62 = 'value62';
  const var63 = 'value63';
  const var64 = 'value64';
  const var65 = 'value65';
  const var66 = 'value66';
  const var67 = 'value67';
  const var68 = 'value68';
  const var69 = 'value69';
  const var70 = 'value70';
  const var71 = 'value71';
  const var72 = 'value72';
  const var73 = 'value73';
  const var74 = 'value74';
  const var75 = 'value75';
  const var76 = 'value76';
  const var77 = 'value77';
  const var78 = 'value78';
  const var79 = 'value79';
  const var80 = 'value80';
  const var81 = 'value81';
  const var82 = 'value82';
  const var83 = 'value83';
  const var84 = 'value84';
  const var85 = 'value85';
  const var86 = 'value86';
  const var87 = 'value87';
  const var88 = 'value88';
  const var89 = 'value89';
  const var90 = 'value90';
  const var91 = 'value91';
  const var92 = 'value92';
  const var93 = 'value93';
  const var94 = 'value94';
  const var95 = 'value95';
  const var96 = 'value96';
  const var97 = 'value97';
  const var98 = 'value98';
  const var99 = 'value99';
  const var100 = 'value100';
  const var101 = 'value101';
  const var102 = 'value102';
  const var103 = 'value103';
  const var104 = 'value104';
  const var105 = 'value105';

  return <div>Large Component</div>;
};

export default LargeComponent;
"#
    }
    
    /// A file with unused imports
    pub fn file_with_unused_imports() -> &'static str {
        r#"
import React, { useState, useEffect } from 'react'; // useEffect is unused
import { Button } from '@/components/ui/button'; // Button is unused
import { formatDate } from '@/utils/date'; // formatDate is unused
import type { User } from '@/types/user'; // User type is unused

const SimpleComponent = () => {
    const [count, setCount] = useState(0);
    
    return (
        <div>
            <p>Count: {count}</p>
            <button onClick={() => setCount(count + 1)}>
                Increment
            </button>
        </div>
    );
};

export default SimpleComponent;
"#
    }
    
    /// A file with potential memory leaks
    pub fn file_with_memory_issues() -> &'static str {
        r#"
import React, { useEffect, useState } from 'react';

const ProblematicComponent = () => {
    const [data, setData] = useState([]);
    
    useEffect(() => {
        // Memory leak: infinite loop without break condition
        while (true) {
            fetch('/api/data')
                .then(response => response.json())
                .then(newData => setData(newData));
        }
    }, []); // Missing dependency array issues
    
    // Another memory leak pattern
    useEffect(() => {
        const interval = setInterval(() => {
            console.log('This will run forever');
        }, 1000);
        
        // Missing cleanup - interval never cleared
    }, []);
    
    return <div>{data.length}</div>;
};

export default ProblematicComponent;
"#
    }
}

/// Test assertion helpers
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that a command completed successfully (exit code 0)
    pub fn assert_success(result: &std::process::Output) {
        if !result.status.success() {
            panic!(
                "Command failed with exit code {:?}\nStdout: {}\nStderr: {}",
                result.status.code(),
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
        }
    }
    
    /// Assert that a command failed with expected exit code
    pub fn assert_failure(result: &std::process::Output, expected_code: Option<i32>) {
        if result.status.success() {
            panic!("Expected command to fail, but it succeeded");
        }
        
        if let Some(code) = expected_code {
            assert_eq!(result.status.code(), Some(code));
        }
    }
    
    /// Assert that JSON output contains expected fields
    pub fn assert_json_structure(json_str: &str, expected_fields: &[&str]) {
        let value: serde_json::Value = serde_json::from_str(json_str)
            .expect("Failed to parse JSON output");
        
        for field in expected_fields {
            assert!(
                value.get(field).is_some(),
                "JSON output missing expected field: {}",
                field
            );
        }
    }
    
    /// Assert that output contains expected text
    pub fn assert_output_contains(output: &str, expected: &str) {
        assert!(
            output.contains(expected),
            "Output does not contain expected text: '{}'\nActual output: {}",
            expected,
            output
        );
    }
    
    /// Assert that output does not contain text
    pub fn assert_output_not_contains(output: &str, unexpected: &str) {
        assert!(
            !output.contains(unexpected),
            "Output contains unexpected text: '{}'\nActual output: {}",
            unexpected,
            output
        );
    }
}

/// Utilities for running commands in tests
pub struct CommandRunner;

impl CommandRunner {
    /// Run a sniff command with arguments from a specific directory
    pub fn run_sniff_command_in_dir<P: AsRef<std::path::Path>>(working_dir: P, args: &[&str]) -> Result<std::process::Output> {
        // Find project root that contains Cargo.toml
        // We need to find the sniff-check project root, not the temporary test directory
        
        // First, try to get the original cargo manifest directory from env
        let mut project_root = None;
        
        // Look from the test executable location (this should be in target/debug/deps)
        if let Ok(exe_path) = std::env::current_exe() {
            for ancestor in exe_path.ancestors() {
                if ancestor.join("Cargo.toml").exists() && 
                   ancestor.file_name().map(|n| n.to_string_lossy()) == Some("sniff-check".into()) {
                    project_root = Some(ancestor.to_path_buf());
                    break;
                }
            }
        }
        
        // Fallback: look upwards from current directory
        if project_root.is_none() {
            let start_search = std::env::current_dir()?;
            for ancestor in start_search.ancestors() {
                if ancestor.join("Cargo.toml").exists() {
                    // Check if this is the sniff-check project by looking for our specific files
                    if ancestor.join("src").join("main.rs").exists() && 
                       ancestor.join("src").join("commands").exists() {
                        project_root = Some(ancestor.to_path_buf());
                        break;
                    }
                }
            }
        }
        
        // Final fallback: use CARGO_MANIFEST_DIR if set
        if project_root.is_none() {
            if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
                let manifest_path = std::path::PathBuf::from(manifest_dir);
                if manifest_path.join("Cargo.toml").exists() {
                    project_root = Some(manifest_path);
                }
            }
        }
        
        let project_root = project_root
            .ok_or_else(|| anyhow::anyhow!("Could not find sniff-check project root with Cargo.toml"))?;
        
        // First build the binary if needed
        std::process::Command::new("cargo")
            .current_dir(&project_root)
            .args(&["build", "--release"])
            .output()?;
            
        // Run the binary directly from the working directory
        let binary_path = project_root.join("target/release/sniff");
        let output = std::process::Command::new(&binary_path)
            .current_dir(&working_dir)
            .args(args)
            .output()?;
        Ok(output)
    }
    
    /// Run a sniff command with arguments
    pub fn run_sniff_command(args: &[&str]) -> Result<std::process::Output> {
        Self::run_sniff_command_in_dir(".", args)
    }
    
    /// Run a sniff command with JSON output
    pub fn run_sniff_json(command: &str, extra_args: &[&str]) -> Result<serde_json::Value> {
        let mut args = vec!["--json", command];
        args.extend(extra_args);
        
        let output = Self::run_sniff_command(&args)?;
        TestAssertions::assert_success(&output);
        
        let stdout = String::from_utf8(output.stdout)?;
        let json: serde_json::Value = serde_json::from_str(&stdout)?;
        Ok(json)
    }
}