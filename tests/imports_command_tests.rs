/// Integration tests for the imports command
mod common;

use common::{TestProject, SampleFiles, TestAssertions, CommandRunner};
use anyhow::Result;

#[test]
fn test_imports_command_finds_unused_imports() -> Result<()> {
    let project = TestProject::new()?;

    project.create_ts_file("components/SimpleComponent", SampleFiles::file_with_unused_imports())?;
    // Stub node_modules so the broken-import check doesn't fire on react
    project.create_dir("node_modules/react")?;

    let output = CommandRunner::run_sniff_command_in_dir(&project.root_path, &["imports"])?;
    TestAssertions::assert_failure(&output, Some(2));

    let stdout = String::from_utf8(output.stdout)?;
    TestAssertions::assert_output_contains(&stdout, "Unused");
    TestAssertions::assert_output_contains(&stdout, "useEffect");
    TestAssertions::assert_output_contains(&stdout, "Button");

    Ok(())
}

#[test]
fn test_imports_command_json_output() -> Result<()> {
    let project = TestProject::new()?;

    // Create a file with unused imports
    project.create_ts_file("components/SimpleComponent", SampleFiles::file_with_unused_imports())?;
    // Stub node_modules so broken-import checks don't fire on well-known packages
    project.create_dir("node_modules/react")?;

    // Run the imports command — expect failure (exit 2) because there are unused imports
    let output = CommandRunner::run_sniff_command_in_dir(&project.root_path, &["--json", "imports"])?;
    let stdout = String::from_utf8(output.stdout)?;

    // Verify JSON structure matches the actual ImportsReport schema
    TestAssertions::assert_json_structure(
        &stdout,
        &["unused_imports", "broken_imports", "summary"]
    );

    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Failed to parse JSON output");

    assert!(json["unused_imports"].as_array().unwrap().len() > 0, "expected unused imports");
    assert!(json["summary"]["unused_imports"].as_u64().unwrap() > 0);

    Ok(())
}

#[test]
fn test_imports_command_clean_file() -> Result<()> {
    let project = TestProject::new()?;

    // Stub node_modules so broken-import checks don't fire on well-known packages
    project.create_dir("node_modules/react")?;

    // Create a file where every import is explicitly used (no JSX React namespace ambiguity)
    project.create_ts_file("components/CleanComponent", r#"
import { useState, useCallback } from 'react';

const CleanComponent = () => {
    const [count, setCount] = useState(0);
    const increment = useCallback(() => setCount(c => c + 1), []);
    return { count, increment };
};

export default CleanComponent;
"#)?;

    let output = CommandRunner::run_sniff_command_in_dir(&project.root_path, &["imports"])?;
    TestAssertions::assert_success(&output);

    let stdout = String::from_utf8(output.stdout)?;
    TestAssertions::assert_output_contains(&stdout, "No import issues found");
    
    Ok(())
}

#[test]
fn test_imports_command_mixed_files() -> Result<()> {
    let project = TestProject::new()?;

    // Stub node_modules so broken-import checks don't fire on well-known packages
    project.create_dir("node_modules/react")?;

    // Create one file with unused imports
    project.create_ts_file("components/BadComponent", SampleFiles::file_with_unused_imports())?;

    // Create one clean file
    project.create_ts_file("components/GoodComponent", r#"
import React from 'react';

const GoodComponent = () => {
    return React.createElement('div', null, 'Good Component');
};

export default GoodComponent;
"#)?;

    let output = CommandRunner::run_sniff_command_in_dir(&project.root_path, &["imports"])?;
    TestAssertions::assert_failure(&output, Some(2));
    
    let stdout = String::from_utf8(output.stdout)?;
    TestAssertions::assert_output_contains(&stdout, "BadComponent.ts");
    TestAssertions::assert_output_not_contains(&stdout, "GoodComponent.ts");
    
    Ok(())
}