use crate::support::kill_all;
use serial_test::serial;
use support::{pass, run, wait_for_stdout};

mod support;

#[tokio::test]
#[serial(piggui)]
#[serial]
async fn two_instances_run() {
    kill_all("piggui");
    let mut piggui = run("piggui", vec![], None);

    wait_for_stdout(&mut piggui, "Connected to hardware", Some("Error: "));

    // Start a second instance - which should exit with an error (not success)
    let mut piggui2 = run("piggui", vec![], None);

    match piggui2.try_wait() {
        Ok(Some(_status)) => panic!("Second instance should not exit"),
        Ok(None) => (),
        Err(_) => {
            println!("Second instance running");
            wait_for_stdout(&mut piggui2, "Error:", Some("Connected to hardware"));
        }
    }

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    pass(&mut piggui);
    pass(&mut piggui2);
}
