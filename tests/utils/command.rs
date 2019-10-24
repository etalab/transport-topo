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
}
