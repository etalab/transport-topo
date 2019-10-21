use docker_compose::DockerComposition;

fn init_log() {
    if std::env::var("RUST_LOG").is_err() {
        let mut builder = pretty_env_logger::formatted_builder();
        builder.filter(None, log::LevelFilter::Info);
        builder.init();
    } else {
        pretty_env_logger::init();
    }
}

fn get_sparql_endpoint(c: &DockerComposition) -> String {
    let port = c
        .port("wdqs-proxy", 80)
        .expect("no port found for wdqs-proxy");
    format!("http://localhost:{port}/bigdata/sparql", port = port)
}

fn get_api_endpoint(c: &DockerComposition) -> String {
    let port = c
        .port("wikibase", 80)
        .expect("impossible to get wikibase port");
    format!("http://localhost:{port}/api.php", port = port)
}

// Check if wdqs is up. for this we do a http query
fn check_wqs(c: &DockerComposition) -> bool {
    let response = reqwest::get(&get_sparql_endpoint(c)).expect("invalid query");

    response.error_for_status().is_ok()
}

fn check_wikibase(c: &DockerComposition) -> bool {
    reqwest::get(&get_api_endpoint(c))
        .and_then(|r| r.error_for_status())
        .is_ok()
}

pub struct DockerContainerWrapper {
    pub docker_compose: DockerComposition,
    pub api_endpoint: String,
    pub sparql_endpoint: String,
}

impl DockerContainerWrapper {
    pub fn new() -> Self {
        init_log();
        let docker_compose = DockerComposition::builder()
            .check(check_wqs)
            .check(check_wikibase)
            .timeout(std::time::Duration::from_secs(5 * 60))
            .build("tests/minimal-docker-compose.yml")
            .expect("unable to run docker compose");

        Self {
            api_endpoint: get_api_endpoint(&docker_compose),
            sparql_endpoint: get_sparql_endpoint(&docker_compose),
            docker_compose,
        }
    }
}

impl std::ops::Deref for DockerContainerWrapper {
    type Target = DockerComposition;
    fn deref(&self) -> &Self::Target {
        &self.docker_compose
    }
}
