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
