use log::{error, info, warn};
use structopt::StructOpt;
use transitwiki::api_client::ObjectType;
use transitwiki::Client;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Configuration file used to define wikidata endpoints, properties and items’ id
    #[structopt(short, long, default_value = "config.toml")]
    config: String,

    /// Identifier of the producer.
    /// Must be an instance (P6) of http://wiki.transport.data.gouv.fr/wiki/Item:Q16
    /// The identifier must be in the form Qxxxx
    /// Otherwise we will search a producer by the name
    #[structopt(short, long)]
    producer: String,

    /// The GTFS file from which we want to import the lines
    #[structopt(short = "i", long = "input-gtfs")]
    gtfs_filename: String,
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
    let mut client = Client::from_config_file(&opt.config).unwrap();

    if opt.producer.starts_with('Q') {
        info!("Searching the producer by id");
        let producer_label = client
            .api
            .get_label(&opt.producer)
            .expect("unable to find producer");
        info!("Found the producer “{}”", &producer_label);
        info!("Starting the importation of lines");
        client
            .import_lines(&opt.gtfs_filename, &opt.producer, &producer_label)
            .expect("unable to import");
    } else {
        info!("Searching the producer by name");
        match client.api.find_entity_id(ObjectType::Item, &opt.producer) {
            Ok(None) => warn!("We found no producer with the name {}", opt.producer),
            Ok(Some(id)) => info!("The following item match the search {}", id),
            Err(error) => error!("Could not find the entity by name: {}", error),
        }
    }
}
