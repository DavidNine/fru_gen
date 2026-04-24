use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_help_message() {
    let mut cmd = Command::cargo_bin("fru_gen").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("FRU_Gen"))
        .stdout(predicate::str::contains("USAGE:"));
}

#[test]
fn test_build_config() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test_config.toml");
    let mut cmd = Command::cargo_bin("fru_gen").unwrap();
    cmd.arg("-b")
        .arg(config_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Build config file"));

    assert!(config_path.exists());
    let content = fs::read_to_string(config_path).unwrap();
    assert!(content.contains("Chassis_type"));
    assert!(content.contains("Board_Manufacturer"));
}

#[test]
fn test_generate_binary_from_toml() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test.toml");
    
    // First generate a template
    let mut build_cmd = Command::cargo_bin("fru_gen").unwrap();
    build_cmd.arg("-b").arg(config_path.to_str().unwrap()).assert().success();

    let output_path = dir.path().join("output.bin");
    let mut cmd = Command::cargo_bin("fru_gen").unwrap();
    cmd.arg("-r").arg(config_path.to_str().unwrap())
       .arg("-o").arg(output_path.to_str().unwrap())
       .arg("--size").arg("1024")
       .assert()
       .success()
       .stdout(predicate::str::contains("Generate fru file"));

    assert!(output_path.exists());
    let metadata = fs::metadata(output_path).unwrap();
    assert_eq!(metadata.len(), 1024);
}

#[test]
fn test_generate_binary_size_too_small() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test_small.toml");
    
    // First generate a template
    let mut build_cmd = Command::cargo_bin("fru_gen").unwrap();
    build_cmd.arg("-b").arg(config_path.to_str().unwrap()).assert().success();

    let output_path = dir.path().join("output_small.bin");
    let mut cmd = Command::cargo_bin("fru_gen").unwrap();
    
    // The default template results in ~600 bytes. 512 should trigger the panic.
    cmd.arg("-r").arg(config_path.to_str().unwrap())
       .arg("-o").arg(output_path.to_str().unwrap())
       .arg("--size").arg("512")
       .assert()
       .failure()
       .stderr(predicate::str::contains("Error: fru data total size exceed limitation"));
}

#[test]
fn test_invalid_config_path() {
    let mut cmd = Command::cargo_bin("fru_gen").unwrap();
    cmd.arg("-r").arg("non_existent_file.toml")
       .assert()
       .failure()
       .stderr(predicate::str::contains("Configuration file 'non_existent_file.toml' not found"));
}
