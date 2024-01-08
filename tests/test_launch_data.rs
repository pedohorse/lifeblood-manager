use core::panic;
use lifeblood_manager::{InstallationsData, LaunchControlData};
use std::matches;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[test]
fn test_launchedprocess_run_wait_till_finishes() {
    launch_test_helper(
        if cfg!(unix) {
            "./proc_exit_clean"
        } else {
            "./proc_exit_clean.cmd"
        },
        vec![],
        Ok(Some(0)),
        -1.,
    );
}

#[test]
fn test_launchedprocess_run_wait_till_finishes_err() {
    launch_test_helper(
        if cfg!(unix) {
            "./proc_exit_1"
        } else {
            "./proc_exit_1.cmd"
        },
        vec![],
        Ok(Some(1)),
        -1.,
    );
}

#[test]
fn test_launchedprocess_run_wait_till_finishes_arg() {
    for i in [1, 2, 3, 5, 7, 12] {
        launch_test_helper(
            if cfg!(unix) {
                "./proc_exit_arg"
            } else {
                "./proc_exit_arg.cmd"
            },
            vec![&i.to_string()],
            Ok(Some(i)),
            -1.,
        );
    }
}

#[test]
fn test_launchedprocess_run_terminate() {
    launch_test_helper(
        if cfg!(unix) {
            "./proc_exit_clean"
        } else {
            "./proc_exit_clean.cmd"
        },
        vec![],
        if cfg!(unix) {
            Ok(None) // on unix return code is None when killed by signal
        } else {
            Ok(Some(0)) // on windows return code is normal
        },
        0.5,
    );
}

fn launch_test_helper(
    program: &str,
    args: Vec<&str>,
    expected_result: Result<Option<i32>, ()>,
    send_term_after: f32,
) {
    let installs =
        if let Ok(x) = InstallationsData::from_dir(PathBuf::from("./tests/data/l_struct1")) {
            Arc::new(Mutex::new(x))
        } else {
            panic!("structure was not parsed!");
        };

    let label = format!("foo: {}", program);
    let mut launch_data = LaunchControlData::new(Some(&installs), &label, program, args, None);

    assert_eq!(label, launch_data.command_label());
    assert_eq!(program, launch_data.command());
    assert!(launch_data.is_current_installation_set());

    assert!(!launch_data.is_process_running());
    if let Err(e) = launch_data.start_process() {
        panic!("failed to start test process: {:?}", e);
    }
    // sure we rely here on process not finishing yet, but process should take whole 2 sec to finish
    assert!(launch_data.is_process_running());
    assert!(matches!(launch_data.try_wait(), Ok(None)));

    let mut passed: f32 = 0.;
    let mut signalled = send_term_after < 0.;
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(500));
        passed += 0.5;
        if !signalled && passed > send_term_after {
            launch_data.process().unwrap().send_terminate_signal().unwrap_or_else(|e| {panic!("error terminating! {:?}", e)});
            signalled = true;
        }
        match launch_data.try_wait() {
            Err(e) => {
                panic!("error {:?}", e);
            }
            Ok(Some(res)) => {
                assert!(!launch_data.is_process_running());
                let exp_code = if let Ok(c) = expected_result {
                    c
                } else {
                    panic!("expected to have an error!");
                };
                assert_eq!(exp_code, res.code());
                assert_eq!(exp_code, launch_data.last_run_exit_code());
                assert!(launch_data.is_current_installation_set());
                assert!(matches!(launch_data.wait(), Err(_))); // consequetive waits are not supposed to work on control data.
                assert!(matches!(launch_data.try_wait(), Err(_)));
                return;
            }
            _ => {
                assert!(launch_data.is_process_running());
                continue;
            }
        }
    }
    panic!("process did not finish in reasonable time!");
}
