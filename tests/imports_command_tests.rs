/// Integration tests for the imports command
mod common;

use common::{TestProject, SampleFiles, TestAssertions, CommandRunner};
use anyhow::Result;

#[test]
fn test_imports_command_finds_unused_imports() -> Result<()> {
    let project = TestProject::new()?;
    
    // Create a file with unused imports
    project.create_ts_file("components/SimpleComponent", SampleFiles::file_with_unused_imports())?;
    
    // Change to project directory
    std::env::set_current_dir(&project.root_path)?;
    
    // Run the imports command
    let output = CommandRunner::run_sniff_command(&["imports"])?;
    TestAssertions::assert_failure(&output, Some(2)); // Should exit with validation failed code
    
    let stdout = String::from_utf8(output.stdout)?;
    TestAssertions::assert_output_contains(&stdout, "unused import");
    TestAssertions::assert_output_contains(&stdout, "useEffect");
    TestAssertions::assert_output_contains(&stdout, "Button");
    
    Ok(())
}

#[test]
fn test_imports_command_json_output() -> Result<()> {
    let project = TestProject::new()?;
    
    // Create a file with unused imports
    project.create_ts_file("components/SimpleComponent", SampleFiles::file_with_unused_imports())?;
    
    // Change to project directory
    std::env::set_current_dir(&project.root_path)?;
    
    // Run the imports command with JSON output
    let json = CommandRunner::run_sniff_json("imports", &[])?;
    
    // Verify JSON structure
    TestAssertions::assert_json_structure(
        &json.to_string(),
        &["command", "timestamp", "version", "data", "summary"]
    );
    
    // Verify command-specific data
    assert_eq!(json["command"], "imports");
    assert_eq!(json["summary"]["status"], "warning");
    assert!(json["data"]["unused_imports"].as_array().unwrap().len() > 0);
    
    Ok(())
}

#[test]
fn test_imports_command_clean_file() -> Result<()> {
    let project = TestProject::new()?;
    
    // Create a file with all imports used
    project.create_ts_file("components/CleanComponent", r#"
import React, { useState } from 'react';
import { Button } from '@/components/ui/button';

const CleanComponent = () => {
    const [count, setCount] = useState(0);
    
    return (
        <div>
            <p>Count: {count}</p>
            <Button onClick={() => setCount(count + 1)}>
                Increment
            </Button>
        </div>
    );
};

export default CleanComponent;
"#)?;
    
    // Change to project directory
    std::env::set_current_dir(&project.root_path)?;
    
    // Run the imports command
    let output = CommandRunner::run_sniff_command(&["imports"])?;
    TestAssertions::assert_success(&output);
    
    let stdout = String::from_utf8(output.stdout)?;
    TestAssertions::assert_output_contains(&stdout, "No unused imports found");
    
    Ok(())
}

#[test]
fn test_imports_command_mixed_files() -> Result<()> {
    let project = TestProject::new()?;
    
    // Create one file with unused imports
    project.create_ts_file("components/BadComponent", SampleFiles::file_with_unused_imports())?;
    
    // Create one clean file
    project.create_ts_file("components/GoodComponent", r#"
import React from 'react';

const GoodComponent = () => {
    return <div>Good Component</div>;
};

export default GoodComponent;
"#)?;
    
    // Change to project directory
    std::env::set_current_dir(&project.root_path)?;
    
    // Run the imports command
    let output = CommandRunner::run_sniff_command(&["imports"])?;
    TestAssertions::assert_failure(&output, Some(2)); // Should fail due to unused imports
    
    let stdout = String::from_utf8(output.stdout)?;
    TestAssertions::assert_output_contains(&stdout, "BadComponent.ts");
    TestAssertions::assert_output_not_contains(&stdout, "GoodComponent.ts");
    
    Ok(())
}