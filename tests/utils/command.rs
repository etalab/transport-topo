use assert_cmd::cargo::CommandCargoExt;

pub fn unchecked_run(target: &str, args: &[&str]) -> std::process::ExitStatus {
    log::info!("running {} {:?}", target, args);
    std::process::Command::cargo_bin(target)
        .unwrap()
        .args(args)
        .status()
        .unwrap()
}

pub fn run(target: &str, args: &[&str]) {
    let status = unchecked_run(target, args);

    assert!(status.success(), "`{}` failed {}", target, &status);

    // TODO remove this wait
    // The graph qb is asyncronously loaded, so the graphql query need some time
    // before being able to query the data
    // we need to find a way to trigger a refresh
    log::info!("waiting a bit to let blazegraph refresh its data");
    std::thread::sleep(std::time::Duration::from_secs(5));
}
