/// Test to ensure the version in README.md matches the version in Cargo.toml
#[test]
fn readme_version_matches_cargo_toml() {
    let cargo_toml_content =
        std::fs::read_to_string("../Cargo.toml").expect("Failed to read Cargo.toml");
    let readme_content = std::fs::read_to_string("../README.md").expect("Failed to read README.md");

    // Extract version from Cargo.toml [package] section
    let mut in_package_section = false;
    let mut cargo_version = None;

    for line in cargo_toml_content.lines() {
        let trimmed = line.trim();

        if trimmed == "[package]" {
            in_package_section = true;
            continue;
        }

        // Exit package section only when encountering a new TOML section (not array brackets)
        if trimmed.starts_with('[') && !trimmed.contains('=') && trimmed != "[package]" {
            in_package_section = false;
        }

        if in_package_section && trimmed.starts_with("version = ") {
            cargo_version = trimmed
                .split('=')
                .nth(1)
                .map(|v| v.trim().trim_matches('"').to_string());
            break;
        }
    }

    let cargo_version =
        cargo_version.expect("Failed to find version in [package] section of Cargo.toml");

    // Check if the version appears in README.md as vX.Y.Z in the pre-commit section
    let expected_version_tag = format!("v{}", cargo_version);

    // Look for the version in the pre-commit configuration section
    // We search for a pattern that includes both the repo and rev to ensure we're in the right context
    const CONTEXT_WINDOW_SIZE: usize = 5;
    let in_pre_commit_section = readme_content
        .lines()
        .collect::<Vec<_>>()
        .windows(CONTEXT_WINDOW_SIZE)
        .any(|window| {
            let window_text = window.join("\n");
            window_text.contains("mondeja/hledger-fmt")
                && window_text.contains(&format!("rev: {}", expected_version_tag))
        });

    assert!(
        in_pre_commit_section,
        "README.md should contain 'rev: {}' near 'mondeja/hledger-fmt' in the pre-commit section, but it was not found.\n\
         Current Cargo.toml version: {}\n\
         Please update the README.md pre-commit section with the correct version.",
        expected_version_tag,
        cargo_version
    );
}
