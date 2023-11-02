use lifeblood_manager::InstallationsData;

#[cfg(unix)]
#[test]
fn test_read_unix_struct1() {
    use std::path::PathBuf;

    let ins = if let Ok(x) = InstallationsData::from_dir(PathBuf::from("./tests/data/u_struct1")) {
        x
    } else {
        panic!("structure was not parsed!");
    };

    assert_eq!(
        PathBuf::from("./tests/data/u_struct1")
            .canonicalize()
            .unwrap(),
        ins.base_path(),
        "base path wrong"
    );
    assert_eq!(3, ins.version_count());
    assert_eq!("hash2", ins.version(0).unwrap().source_commit_hash());
    assert_eq!("hash1", ins.version(1).unwrap().source_commit_hash());
    assert_eq!("hash3", ins.version(2).unwrap().source_commit_hash());
    assert_eq!(0, ins.current_version_index());
}

#[cfg(unix)]
#[test]
fn test_read_unix_struct2() {
    use std::path::PathBuf;

    let expected = PathBuf::from("./tests/data/u_struct2");
    let actual = PathBuf::from("./tests/data/u_struct2_act");

    if actual.exists() {
        std::fs::remove_dir_all(&actual).unwrap();
    }
    let mut options = fs_extra::dir::CopyOptions::new();

    options.copy_inside = true;
    fs_extra::dir::copy(&expected, &actual, &options).unwrap();
    // hack... TODO: learn how to copy symlinks
    std::fs::remove_dir_all("./tests/data/u_struct2_act/current").unwrap();
    std::os::unix::fs::symlink("hash2", "./tests/data/u_struct2_act/current").unwrap();

    let mut ins =
        if let Ok(x) = InstallationsData::from_dir(PathBuf::from("./tests/data/u_struct2_act")) {
            x
        } else {
            panic!("structure was not parsed!");
        };

    assert_eq!(0, ins.current_version_index());

    let current = PathBuf::from("./tests/data/u_struct2_act/current");

    ins.make_version_current(1).unwrap();
    assert_eq!(1, ins.current_version_index());

    assert!(current.is_symlink());
    assert_eq!("hash1", current.read_link().unwrap().file_name().unwrap(), "incorrect current set");

    ins.make_version_current(2).unwrap();
    assert_eq!(2, ins.current_version_index());

    assert!(current.is_symlink());
    assert_eq!("hash3", current.read_link().unwrap().file_name().unwrap(), "incorrect current set");

    ins.make_version_current(0).unwrap();
    assert_eq!(0, ins.current_version_index());

    assert!(current.is_symlink());
    assert_eq!("hash2", current.read_link().unwrap().file_name().unwrap(), "incorrect current set");
}
