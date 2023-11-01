use lifeblood_manager::InstallationsData;


#[cfg(unix)]
#[test]
fn test_read_unix_struct1() {
    use std::path::PathBuf;

    let ins = if let Ok(x) = InstallationsData::from_dir(PathBuf::from("./tests/data/u_struct1")) { x } else {
        panic!("structure was not parsed!");
    };

    assert_eq!(PathBuf::from("./tests/data/u_struct1").canonicalize().unwrap(), ins.base_path(), "base path wrong");
    assert_eq!(3, ins.version_count());
    assert_eq!("hash2", ins.version(0).unwrap().source_commit_hash());
    assert_eq!("hash1", ins.version(1).unwrap().source_commit_hash());
    assert_eq!("hash3", ins.version(2).unwrap().source_commit_hash());
}
