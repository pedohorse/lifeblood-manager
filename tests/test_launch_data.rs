use lifeblood_manager::{LaunchControlData, LaunchedProcess, InstallationsData};
use std::path::PathBuf;
use std::matches;
use std::thread;
use std::time::Duration;


#[test]
fn test_launchedprocess_run_wait_till_finishes() {
    let installs = if let Ok(x) = InstallationsData::from_dir(PathBuf::from("./tests/data/l_struct1")) {
        x
    } else {
        panic!("structure was not parsed!");
    };

    let mut proc = match LaunchedProcess::new(installs.base_path(), "./proc_exit_clean", &vec![]) {
        Ok(p) => p,
        Err(e) => {
            panic!("error happened! {:?}", e);
        }
    };

    assert_eq!(installs.base_path(), proc.base_path());
    // sure we rely here on process not finishing yet, but process should take whole 2 sec to finish
    assert!(matches!(proc.try_wait(), Ok(None)));

    for _ in 0..10 {
        thread::sleep(Duration::from_millis(500));
        match proc.try_wait() {
            Err(e) => {
                panic!("error {:?}", e);
            }
            Ok(Some(res)) => {
                assert!(res.success());
                assert_eq!(installs.base_path(), proc.base_path());
                assert!(matches!(proc.try_wait(), Ok(Some(x)) if x == res));  // consequetive try_wait should return same shit
                assert!(matches!(proc.wait(), Ok(x) if x == res));
                return;
            }
            _ => continue
        }
    }
    panic!("process did not finish in reasonable time!");
}