/// Integration tests for the types command
mod common;

use common::{TestProject, SampleFiles, TestAssertions, CommandRunner};
use anyhow::Result;

#[test]
fn test_types_command_finds_type_issues() -> Result<()> {
    let project = TestProject::new()?;

    project.create_ts_file("components/UserCard", SampleFiles::component_with_type_issues())?;
    project.create_ts_file("components/CleanComponent", r#"
interface Props {
    title: string;
    count: number;
}

const CleanComponent = ({ title, count }: Props) => {
    return title + count;
};

export default CleanComponent;
"#)?;

    let output = CommandRunner::run_sniff_command_in_dir(&project.root_path, &["types"])?;
    TestAssertions::assert_failure(&output, Some(2));

    let stdout = String::from_utf8(output.stdout)?;
    TestAssertions::assert_output_contains(&stdout, "UserCard.ts");
    TestAssertions::assert_output_contains(&stdout, "any");

    Ok(())
}

#[test]
fn test_types_command_json_output() -> Result<()> {
    let project = TestProject::new()?;

    project.create_ts_file("components/UserCard", SampleFiles::component_with_type_issues())?;

    let output = CommandRunner::run_sniff_command_in_dir(&project.root_path, &["--json", "types"])?;
    let stdout = String::from_utf8(output.stdout)?;

    // types uses plain TypeScriptReport: { issues, summary }
    TestAssertions::assert_json_structure(&stdout, &["issues", "summary"]);

    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Failed to parse JSON output");
    assert!(json["issues"].as_array().unwrap().len() > 0, "expected type issues");
    assert!(json["summary"]["any_usage_count"].as_u64().unwrap() > 0);

    Ok(())
}

#[test]
fn test_types_command_clean_project() -> Result<()> {
    let project = TestProject::new()?;

    project.create_ts_file("components/CleanComponent", r#"
interface ButtonProps {
    label: string;
    onClick: () => void;
    disabled?: boolean;
}

const Button = ({ label, onClick, disabled = false }: ButtonProps) => {
    return label + disabled + onClick;
};

export default Button;
"#)?;

    let output = CommandRunner::run_sniff_command_in_dir(&project.root_path, &["types"])?;
    TestAssertions::assert_success(&output);

    let stdout = String::from_utf8(output.stdout)?;
    TestAssertions::assert_output_contains(&stdout, "No issues found");

    Ok(())
}

#[test]
fn test_types_command_detects_ts_ignore() -> Result<()> {
    let project = TestProject::new()?;

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

    let output = CommandRunner::run_sniff_command_in_dir(&project.root_path, &["types"])?;
    TestAssertions::assert_failure(&output, Some(2));

    let stdout = String::from_utf8(output.stdout)?;
    TestAssertions::assert_output_contains(&stdout, "@ts-ignore");

    Ok(())
}
