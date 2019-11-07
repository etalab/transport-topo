use structopt::StructOpt;
use transit_topo::{api_client, Client};

#[derive(StructOpt, Debug)]
#[structopt(name = "import-gtfs")]
enum Opt {
    #[structopt(name = "create")]
    Create {
        /// Label of the new producer. No other producer can have this label.
        label: String,

        /// Identifier of the topo id property
        #[structopt(short, long, default_value = "P1")]
        topo_id_id: String,

        /// Endpoint of the wikibase api
        #[structopt(short, long)]
        api: String,

        /// Endpoint of the sparql query serive
        #[structopt(short, long)]
        sparql: String,
    },
}

fn create_producer(
    label: &str,
    topo_id_id: &str,
    api: &str,
    sparql: &str,
) -> Result<String, anyhow::Error> {
    let client = Client::new(api, sparql, topo_id_id)?;

    // We check that there is not yet a producer with this label
    match client.sparql.get_producer_id(label)? {
        Some(id) => {
            log::info!("producer {} already exists with id {}", label, id);
            Ok(id.to_owned())
        }
        None => {
            log::info!("no producer \"{}\" exists, creating one", label);
            let entities_id = &client.sparql.config;
            let claims = [api_client::claim_item(
                &entities_id.properties.instance_of,
                &entities_id.items.producer,
            )];
            let id = client.api.create_item(label, &claims)?;
            log::info!("creating producer \"{}\" with id {}", label, id);
            Ok(id.to_owned())
        }
    }
}

fn main() {
    transit_topo::log::init();

    let opt = Opt::from_args();

    match opt {
        Opt::Create {
            label,
            topo_id_id,
            api,
            sparql,
        } => {
            let id = create_producer(&label, &topo_id_id, &api, &sparql)
                .expect("impossible to create producer");
            log::info!("producer id: {}", id);
        }
    }
}
