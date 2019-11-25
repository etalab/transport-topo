use structopt::StructOpt;
use transit_topo::GtfsImporter;

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
    #[structopt(short, long, default_value = "http://localhost:8181/api.php")]
    api: String,

    /// Endpoint of the sparql query serive
    #[structopt(short, long, default_value = "http://localhost:8989/bigdata/sparql")]
    sparql: String,
}

fn main() {
    transit_topo::log::init();

    let opt = Opt::from_args();
    let importer = GtfsImporter::new(&opt.api, &opt.sparql, &opt.topo_id_id).unwrap();

    log::info!("Searching the producer by id");
    let producer_label = importer
        .query
        .get_producer_label(&opt.producer)
        .expect("unable to search for producer")
        .unwrap_or_else(|| panic!("no producer with id {}", &opt.producer));
    log::info!("Found the producer “{}”", &producer_label);
    log::info!("Starting the importation of lines");
    importer
        .import_gtfs(&opt.gtfs_filename, &opt.producer, &producer_label)
        .expect("unable to import");
}
