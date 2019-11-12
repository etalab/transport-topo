use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Endpoint of the wikibase api
    #[structopt(short, long)]
    api: String,
    /// Endpoint of the sparql query serive
    #[structopt(short, long)]
    sparql: String,
}

fn main() {
    transit_topo::log::init();

    let opt = Opt::from_args();
    transit_topo::database_initializer::initial_populate(&opt.api, &opt.sparql)
        .expect("impossible to populate wikibase");
}
