/// Integration tests for the types command
mod common;

use common::{TestProject, SampleFiles, TestAssertions, CommandRunner};
use anyhow::Result;

#[test]
fn test_types_command_finds_type_issues() -> Result<()> {
    let project = TestProject::new()?;
    
    // Create a file with type issues
    project.create_ts_file("components/UserCard", SampleFiles::component_with_type_issues())?;
    
    // Create a clean file
    project.create_ts_file("components/CleanComponent", r#"
interface Props {
    title: string;
    count: number;
}

const CleanComponent: React.FC<Props> = ({ title, count }) => {
    return <div>{title}: {count}</div>;
};

export default CleanComponent;
"#)?;
    
    // Change to project directory
    std::env::set_current_dir(&project.root_path)?;
    
    // Run the types command
    let output = CommandRunner::run_sniff_command(&["types"])?;
    TestAssertions::assert_failure(&output, Some(2)); // Should exit with validation failed code
    
    let stdout = String::from_utf8(output.stdout)?;
    TestAssertions::assert_output_contains(&stdout, "UserCard.ts");
    TestAssertions::assert_output_contains(&stdout, "any");
    
    Ok(())
}

#[test]
fn test_types_command_json_output() -> Result<()> {
    let project = TestProject::new()?;
    
    // Create a file with type issues
    project.create_ts_file("components/UserCard", SampleFiles::component_with_type_issues())?;
    
    // Change to project directory
    std::env::set_current_dir(&project.root_path)?;
    
    // Run the types command with JSON output
    let json = CommandRunner::run_sniff_json("types", &[])?;
    
    // Verify JSON structure
    TestAssertions::assert_json_structure(
        &json.to_string(),
        &["command", "timestamp", "version", "data", "summary"]
    );
    
    // Verify command-specific data
    assert_eq!(json["command"], "types");
    assert_eq!(json["summary"]["status"], "warning");
    assert!(json["data"]["issues"].as_array().unwrap().len() > 0);
    
    Ok(())
}

#[test]
fn test_types_command_clean_project() -> Result<()> {
    let project = TestProject::new()?;
    
    // Create only clean TypeScript files
    project.create_ts_file("components/CleanComponent", r#"
interface ButtonProps {
    label: string;
    onClick: () => void;
    disabled?: boolean;
}

const Button: React.FC<ButtonProps> = ({ label, onClick, disabled = false }) => {
    return (
        <button onClick={onClick} disabled={disabled}>
            {label}
        </button>
    );
};

export default Button;
"#)?;
    
    // Change to project directory
    std::env::set_current_dir(&project.root_path)?;
    
    // Run the types command
    let output = CommandRunner::run_sniff_command(&["types"])?;
    TestAssertions::assert_success(&output);
    
    let stdout = String::from_utf8(output.stdout)?;
    TestAssertions::assert_output_contains(&stdout, "No TypeScript issues found");
    
    Ok(())
}

#[test]
fn test_types_command_detects_ts_ignore() -> Result<()> {
    let project = TestProject::new()?;
    
    // Create a file with many @ts-ignore comments
    project.create_ts_file("components/BadComponent", r#"
// @ts-ignore
const badVar: any = something;

// @ts-ignore
const anotherBad = someUndefinedFunction();

// @ts-ignore
const moreBad = yetAnotherFunction();

// @ts-ignore
const evenMore = finalBadFunction();

// @ts-ignore
const tooMany = lastBadFunction();

// @ts-ignore
const wayTooMany = reallyLastBadFunction();

export default BadComponent;
"#)?;
    
    // Change to project directory
    std::env::set_current_dir(&project.root_path)?;
    
    // Run the types command
    let output = CommandRunner::run_sniff_command(&["types"])?;
    TestAssertions::assert_failure(&output, Some(2)); // Should fail due to too many @ts-ignore
    
    let stdout = String::from_utf8(output.stdout)?;
    TestAssertions::assert_output_contains(&stdout, "@ts-ignore");
    
    Ok(())
}