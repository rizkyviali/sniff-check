/// Integration tests for the large command
mod common;

use common::{TestProject, SampleFiles, TestAssertions, CommandRunner};
use anyhow::Result;

#[test]
fn test_large_command_finds_large_files() -> Result<()> {
    let project = TestProject::new()?;

    project.create_ts_file("components/LargeComponent", SampleFiles::large_component())?;
    project.create_ts_file("components/SmallComponent", "export const Small = () => null;")?;

    let output = CommandRunner::run_sniff_command_in_dir(&project.root_path, &["large", "--threshold", "50"])?;
    TestAssertions::assert_failure(&output, Some(3));

    let stdout = String::from_utf8(output.stdout)?;
    TestAssertions::assert_output_contains(&stdout, "LargeComponent.ts");
    TestAssertions::assert_output_not_contains(&stdout, "SmallComponent.ts");

    Ok(())
}

#[test]
fn test_large_command_json_output() -> Result<()> {
    let project = TestProject::new()?;

    project.create_ts_file("components/LargeComponent", SampleFiles::large_component())?;

    let output = CommandRunner::run_sniff_command_in_dir(
        &project.root_path,
        &["--json", "large", "--threshold", "50"],
    )?;
    let stdout = String::from_utf8(output.stdout)?;

    // large uses StandardResponse wrapper: { command, timestamp, version, data, summary }
    TestAssertions::assert_json_structure(&stdout, &["command", "data", "summary"]);

    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Failed to parse JSON output");
    assert_eq!(json["command"], "large");
    assert!(json["summary"]["issues_found"].as_u64().unwrap() > 0, "expected large files");

    Ok(())
}

#[test]
fn test_large_command_no_large_files() -> Result<()> {
    let project = TestProject::new()?;

    project.create_ts_file("components/SmallComponent", "export const Small = () => null;")?;
    project.create_js_file("utils/helper", "export const helper = () => 'help';")?;

    let output = CommandRunner::run_sniff_command_in_dir(
        &project.root_path,
        &["large", "--threshold", "50"],
    )?;
    TestAssertions::assert_success(&output);

    let stdout = String::from_utf8(output.stdout)?;
    TestAssertions::assert_output_contains(&stdout, "No large files found");

    Ok(())
}

#[test]
fn test_large_command_custom_threshold() -> Result<()> {
    let project = TestProject::new()?;

    let medium_content = "const x = 1;\n".repeat(20);
    project.create_ts_file("components/MediumComponent", &medium_content)?;

    let output = CommandRunner::run_sniff_command_in_dir(
        &project.root_path,
        &["large", "--threshold", "10"],
    )?;
    TestAssertions::assert_failure(&output, Some(3));

    let output = CommandRunner::run_sniff_command_in_dir(
        &project.root_path,
        &["large", "--threshold", "50"],
    )?;
    TestAssertions::assert_success(&output);

    Ok(())
}

#[test]
fn test_large_command_quiet_mode() -> Result<()> {
    let project = TestProject::new()?;

    project.create_ts_file("components/LargeComponent", SampleFiles::large_component())?;

    let output = CommandRunner::run_sniff_command_in_dir(
        &project.root_path,
        &["--quiet", "large", "--threshold", "50"],
    )?;

    let stdout = String::from_utf8(output.stdout)?;
    TestAssertions::assert_output_not_contains(&stdout, "🔍 Running large file analysis");

    Ok(())
}