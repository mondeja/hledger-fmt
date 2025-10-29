/// Test to ensure the version in README.md matches the version in Cargo.toml
#[test]
fn readme_version_matches_cargo_toml() {
    let cargo_toml_content = std::fs::read_to_string("Cargo.toml")
        .expect("Failed to read Cargo.toml");
    let readme_content = std::fs::read_to_string("README.md")
        .expect("Failed to read README.md");

    // Extract version from Cargo.toml
    let cargo_version = cargo_toml_content
        .lines()
        .find(|line| line.trim().starts_with("version = "))
        .and_then(|line| {
            line.split('=')
                .nth(1)
                .map(|v| v.trim().trim_matches('"'))
        })
        .expect("Failed to find version in Cargo.toml");

    // Check if the version appears in README.md as vX.Y.Z in the pre-commit section
    let expected_version_tag = format!("v{}", cargo_version);
    
    // Look for the version in the pre-commit configuration section
    let has_version = readme_content.contains(&format!("rev: {}", expected_version_tag));

    assert!(
        has_version,
        "README.md should contain 'rev: {}' in the pre-commit section, but it was not found.\n\
         Current Cargo.toml version: {}\n\
         Please update the README.md pre-commit section with the correct version.",
        expected_version_tag,
        cargo_version
    );
}
