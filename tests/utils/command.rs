use crate::utils::DockerContainerWrapper;
use assert_cmd::cargo::CommandCargoExt;

pub fn run(docker: &DockerContainerWrapper, target: &str, args: &[&str]) {
    let api_port = docker
        .port("wikibase", 80)
        .expect("no port found for wikibase");
    let sparql_port = docker
        .port("wdqs-proxy", 80)
        .expect("no port found for wdqs-proxy");
    let api_endpoint = format!("http://localhost:{port}/api.php", port = api_port);
    let sparql_endpoint = format!("http://localhost:{port}/bigdata/sparql", port = sparql_port);

    log::info!("running {}", target);
    let status = std::process::Command::cargo_bin(target)
        .unwrap()
        .arg("--api")
        .arg(&api_endpoint)
        .arg("--sparql")
        .arg(&sparql_endpoint)
        .args(args)
        .status()
        .unwrap();

    assert!(status.success(), "`{}` failed {}", target, &status);

    // TODO remove this wait
    // The graph qb is asyncronously loaded, so the graphql query need some time
    // before being able to query the data
    // we need to find a way to trigger a refresh
    log::info!("waiting a bit to let blazegraph refresh its data");
    std::thread::sleep(std::time::Duration::from_secs(15));
}
