use std::{fs, io::Write, path::PathBuf};

pub(crate) fn create_test_directory_tree() -> Result<PathBuf, anyhow::Error> {
    let temp_dir = std::env::temp_dir();
    let temp_dir = temp_dir.join(format!("space_{}", uuid::Uuid::new_v4()));
    fs::create_dir(&temp_dir)?;
    let d1 = &temp_dir.join("1");
    fs::create_dir_all(d1)?;
    create_test_file(d1.join("1.1"), 25000)?;
    create_test_file(d1.join("1.2"), 24000)?;
    let d1_3 = &d1.join("1.3");
    fs::create_dir_all(d1_3)?;
    create_test_file(d1_3.join("1.3.1"), 20000)?;
    create_test_file(d1_3.join("1.3.2"), 2000)?;
    create_test_file(d1_3.join("1.3.3"), 1000)?;
    create_test_file(d1.join("1.4"), 22000)?;
    let d1_5 = &d1.join("1.5");
    fs::create_dir_all(d1_5)?;
    create_test_file(d1_5.join("1.5.1"), 6000)?;
    create_test_file(d1_5.join("1.5.2"), 5000)?;
    let d1_5_3 = &d1_5.join("1.5.3");
    fs::create_dir_all(d1_5_3)?;
    create_test_file(d1_5_3.join("1.5.3.1"), 1800)?;
    create_test_file(d1_5_3.join("1.5.3.2"), 1000)?;
    create_test_file(d1_5_3.join("1.5.3.3"), 700)?;
    create_test_file(d1_5_3.join("1.5.3.4"), 500)?;
    let d1_5_3_5 = &d1_5_3.join("1.5.3.5"); // empty directory
    fs::create_dir_all(d1_5_3_5)?;
    create_test_file(d1_5.join("1.5.4"), 2000)?;
    create_test_file(d1_5.join("1.5.5"), 1000)?;
    create_test_file(d1.join("1.6"), 16000)?;
    create_test_file(d1.join("1.7"), 15000)?;
    create_test_file(d1.join("1.8"), 14000)?;
    create_test_file(d1.join("1.9"), 13000)?;

    let d1_10 = &d1.join("1.10"); // directory with one file
    fs::create_dir_all(d1_10)?;
    create_test_file(d1_10.join("1.10.1"), 10000)?;

    create_test_symlink_dir(d1_5_3, &d1.join("1.11"))?;

    let d1_12 = &d1.join("1.12"); // directory with one symbolic link
    fs::create_dir_all(d1_12)?;
    create_test_symlink_dir(d1_10, &d1_12.join("1.12.1"))?;

    Ok(temp_dir)
}

pub(crate) fn create_test_file(path: PathBuf, len: usize) -> anyhow::Result<()> {
    let mut f_1_2 = fs::File::create(path)?;
    let bytes: Vec<u8> = vec![0; len];
    f_1_2.write_all(&bytes.as_slice())?;
    Ok(())
}

pub(crate) fn create_test_symlink_dir(original: &PathBuf, link: &PathBuf) -> anyhow::Result<()> {
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(original, link)?;
    #[cfg(unix)]
    std::os::unix::fs::symlink(original, link)?;
    Ok(())
}

pub(crate) fn delete_test_directory_tree(path: &PathBuf) {
    // This is best effort. Ignore any errors.
    let _ = fs::remove_dir_all(path);
}
