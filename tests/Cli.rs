use std::io::Write;


#[test]
fn test_with_valid_path() {
    // Create temp file
    let mut file = tempfile::NamedTempFile::new().expect("Could not create temporary file");
    writeln!(file, "Test content").expect("Could not write content into temporary file");


    let content = fru_gen::read_config_file(Some(file.path().to_path_buf())).expect("Could not read the regular file");
    assert_eq!(content, "Test content\n");
}

