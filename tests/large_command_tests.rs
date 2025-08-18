/// Integration tests for the large command
mod common;

use common::{TestProject, SampleFiles, TestAssertions, CommandRunner};
use anyhow::Result;

#[test]
fn test_large_command_finds_large_files() -> Result<()> {
    let project = TestProject::new()?;
    
    // Create a large file
    project.create_ts_file("components/LargeComponent", SampleFiles::large_component())?;
    
    // Create a small file
    project.create_ts_file("components/SmallComponent", "export const Small = () => <div>Small</div>;")?;
    
    // Change to project directory
    std::env::set_current_dir(&project.root_path)?;
    
    // Run the large command
    let output = CommandRunner::run_sniff_command(&["large", "--threshold", "50"])?;
    TestAssertions::assert_failure(&output, Some(3)); // Should exit with code 3 (threshold exceeded)
    
    let stdout = String::from_utf8(output.stdout)?;
    TestAssertions::assert_output_contains(&stdout, "LargeComponent.ts");
    TestAssertions::assert_output_not_contains(&stdout, "SmallComponent.ts");
    
    Ok(())
}

#[test]
fn test_large_command_json_output() -> Result<()> {
    let project = TestProject::new()?;
    
    // Create a large file
    project.create_ts_file("components/LargeComponent", SampleFiles::large_component())?;
    
    // Change to project directory
    std::env::set_current_dir(&project.root_path)?;
    
    // Run the large command with JSON output
    let json = CommandRunner::run_sniff_json("large", &["--threshold", "50"])?;
    
    // Verify JSON structure
    TestAssertions::assert_json_structure(
        &json.to_string(),
        &["command", "timestamp", "version", "data", "summary"]
    );
    
    // Verify command-specific data
    assert_eq!(json["command"], "large");
    assert_eq!(json["summary"]["status"], "warning");
    assert!(json["summary"]["issues_found"].as_u64().unwrap() > 0);
    
    Ok(())
}

#[test]
fn test_large_command_no_large_files() -> Result<()> {
    let project = TestProject::new()?;
    
    // Create only small files
    project.create_ts_file("components/SmallComponent", "export const Small = () => <div>Small</div>;")?;
    project.create_js_file("utils/helper", "export const helper = () => 'help';")?;
    
    // Change to project directory
    std::env::set_current_dir(&project.root_path)?;
    
    // Run the large command
    let output = CommandRunner::run_sniff_command(&["large", "--threshold", "50"])?;
    TestAssertions::assert_success(&output);
    
    let stdout = String::from_utf8(output.stdout)?;
    TestAssertions::assert_output_contains(&stdout, "0 large files found");
    
    Ok(())
}

#[test]
fn test_large_command_custom_threshold() -> Result<()> {
    let project = TestProject::new()?;
    
    // Create a medium-sized file (around 20 lines)
    let medium_content = "import React from 'react';\n".repeat(20);
    project.create_ts_file("components/MediumComponent", &medium_content)?;
    
    // Change to project directory
    std::env::set_current_dir(&project.root_path)?;
    
    // Test with low threshold (should find the file)
    let output = CommandRunner::run_sniff_command(&["large", "--threshold", "10"])?;
    TestAssertions::assert_failure(&output, Some(3));
    
    // Test with high threshold (should not find the file)
    let output = CommandRunner::run_sniff_command(&["large", "--threshold", "50"])?;
    TestAssertions::assert_success(&output);
    
    Ok(())
}

#[test]
fn test_large_command_quiet_mode() -> Result<()> {
    let project = TestProject::new()?;
    
    // Create a large file
    project.create_ts_file("components/LargeComponent", SampleFiles::large_component())?;
    
    // Change to project directory
    std::env::set_current_dir(&project.root_path)?;
    
    // Run in quiet mode
    let output = CommandRunner::run_sniff_command(&["--quiet", "large", "--threshold", "50"])?;
    
    let stdout = String::from_utf8(output.stdout)?;
    // In quiet mode, it should still show the summary but less verbose output
    TestAssertions::assert_output_not_contains(&stdout, "🔍 Running large file analysis");
    
    Ok(())
}