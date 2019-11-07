use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Endpoint of the wikibase api
    #[structopt(short, long)]
    api: String,
}

fn main() {
    transit_topo::log::init();

    let opt = Opt::from_args();
    transit_topo::database_initializer::initial_populate(&opt.api, true)
        .expect("impossible to populate wikibase");
}
