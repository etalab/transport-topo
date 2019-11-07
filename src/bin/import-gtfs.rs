use log::{error, info, warn};
use structopt::StructOpt;
use transit_topo::api_client::ObjectType;
use transit_topo::Client;

#[derive(StructOpt, Debug)]
#[structopt(name = "import-gtfs")]
struct Opt {
    /// Identifier of the topo id property
    #[structopt(short, long, default_value = "P1")]
    topo_id_id: String,

    /// Identifier of the producer.
    /// Must be an instance (P6) of http://wiki.transport.data.gouv.fr/wiki/Item:Q16
    /// The identifier must be in the form Qxxxx
    /// Otherwise we will search a producer by the name
    #[structopt(short, long)]
    producer: String,

    /// The GTFS file from which we want to import the lines
    #[structopt(short = "i", long = "input-gtfs")]
    gtfs_filename: String,

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
    let client = Client::new(&opt.api, &opt.sparql, &opt.topo_id_id).unwrap();

    if opt.producer.starts_with('Q') {
        info!("Searching the producer by id");
        let producer_label = client
            .api
            .get_label(&opt.producer)
            .expect("unable to find producer");
        info!("Found the producer “{}”", &producer_label);
        info!("Starting the importation of lines");
        client
            .import_gtfs(&opt.gtfs_filename, &opt.producer, &producer_label)
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
