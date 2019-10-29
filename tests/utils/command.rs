use assert_cmd::cargo::CommandCargoExt;

pub fn run(target: &str, args: &[&str]) {
    log::info!("running {} {:?}", target, args);
    let status = std::process::Command::cargo_bin(target)
        .unwrap()
        .args(args)
        .status()
        .unwrap();

    assert!(status.success(), "`{}` failed {}", target, &status);

    // TODO remove this wait
    // The graph qb is asyncronously loaded, so the graphql query need some time
    // before being able to query the data
    // we need to find a way to trigger a refresh
    log::info!("waiting a bit to let blazegraph refresh its data");
    std::thread::sleep(std::time::Duration::from_secs(5));
}
