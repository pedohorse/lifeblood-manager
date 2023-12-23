use lifeblood_manager::{InstallationsData, LaunchedProcess};
use std::matches;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[test]
fn test_launchedprocess_run_wait_till_finishes() {
    launch_test_helper("./proc_exit_clean", &vec![], Ok(Some(0)), -1.);
}

#[test]
fn test_launchedprocess_run_wait_till_finishes_err() {
    launch_test_helper("./proc_exit_1", &vec![], Ok(Some(1)), -1.);
}

#[test]
fn test_launchedprocess_run_wait_till_finishes_arg() {
    for i in [1, 2, 3, 5, 7, 12] {
        launch_test_helper("./proc_exit_arg", &vec![i.to_string()], Ok(Some(i)), -1.);
    }
}

#[test]
fn test_launchedprocess_run_terminate() {
    launch_test_helper("./proc_exit_clean", &vec![], Ok(None), 0.5);
}

fn launch_test_helper(
    program: &str,
    args: &Vec<String>,
    expected_result: Result<Option<i32>, ()>,
    send_term_after: f32,
) {
    let installs =
        if let Ok(x) = InstallationsData::from_dir(PathBuf::from("./tests/data/l_struct1")) {
            x
        } else {
            panic!("structure was not parsed!");
        };

    let mut proc = match LaunchedProcess::new(installs.base_path(), program, args) {
        Ok(p) => p,
        Err(e) => {
            panic!("error happened! {:?}", e);
        }
    };

    assert_eq!(installs.base_path(), proc.base_path());
    // sure we rely here on process not finishing yet, but process should take whole 2 sec to finish
    assert!(matches!(proc.try_wait(), Ok(None)));

    let mut passed: f32 = 0.;
    let mut signalled = send_term_after < 0.;
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(500));
        passed += 0.5;
        if !signalled && passed > send_term_after {
            proc.send_terminate_signal();
            signalled = true;
        }
        match proc.try_wait() {
            Err(e) => {
                panic!("error {:?}", e);
            }
            Ok(Some(res)) => {
                let exp_code = if let Ok(c) = expected_result {
                    c
                } else {
                    panic!("expected to have an error!");
                };
                assert_eq!(exp_code, res.code());
                assert_eq!(installs.base_path(), proc.base_path());
                assert!(matches!(proc.try_wait(), Ok(Some(x)) if x == res)); // consequetive try_wait should return same shit
                assert!(matches!(proc.wait(), Ok(x) if x == res));
                return;
            }
            _ => continue,
        }
    }
    panic!("process did not finish in reasonable time!");
}
