use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Endpoint of the wikibase api
    #[structopt(short, long)]
    api: String,
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        let mut builder = pretty_env_logger::formatted_builder();
        builder.filter(None, log::LevelFilter::Info);
        builder.init();
    } else {
        pretty_env_logger::init();
    }

    let opt = Opt::from_args();
    transit_topo::database_initializer::initial_populate(&opt.api, true)
        .expect("impossible to populate wikibase");
}
