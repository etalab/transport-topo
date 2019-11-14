use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Endpoint of the wikibase api
    #[structopt(short, long, default_value = "http://localhost:8181/api.php")]
    api: String,
    /// Endpoint of the sparql query serive
    #[structopt(short, long, default_value = "http://localhost:8989/bigdata/sparql")]
    sparql: String,
}

fn main() {
    transit_topo::log::init();

    let opt = Opt::from_args();
    transit_topo::database_initializer::initial_populate(&opt.api, &opt.sparql)
        .expect("impossible to populate wikibase");
}
