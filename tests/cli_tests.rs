#[test]
fn test_cli_examples() {
    trycmd::TestCases::new().case("README.md");
}

#[test]
fn test_cli_list_rolls() {
    trycmd::TestCases::new().case("tests/list-rolls.trycmd");
}

#[test]
fn test_cli_list_frames() {
    trycmd::TestCases::new().case("tests/list-frames.trycmd");
}

#[test]
fn test_cli_tag() {
    trycmd::TestCases::new().case("tests/tag.trycmd");
}

#[test]
fn test_cli_apply_metadata() {
    trycmd::TestCases::new().case("tests/apply-metadata.trycmd");
}
