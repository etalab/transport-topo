use structopt::StructOpt;
use transit_topo::Client;

#[derive(StructOpt, Debug)]
#[structopt(name = "entities")]
enum Opt {
    Search {
        /// Identifier of the topo id property
        #[structopt(short, long, default_value = "P1")]
        topo_id_id: String,

        /// Endpoint of the wikibase api
        #[structopt(short, long, default_value = "http://localhost:8181/api.php")]
        api: String,

        /// Endpoint of the sparql query serive
        #[structopt(short, long, default_value = "http://localhost:8989/bigdata/sparql")]
        sparql: String,

        /// Extra claim with the form P42:foobar. Can be repeated
        #[structopt(short, long = "claim")]
        claims: Vec<String>,
    },
}

fn parse_claims(claims: &[String]) -> Result<Vec<(String, String)>, anyhow::Error> {
    let re = regex::Regex::new(r"^(.*)=(.*)$")?;
    claims
        .iter()
        .map(|claim| {
            let captures = re
                .captures(&claim)
                .ok_or_else(|| anyhow::anyhow!("Could not parse claim {}", claim))?;
            Ok((captures[1].to_owned(), captures[2].to_owned()))
        })
        .collect()
}

fn search(
    topo_id_id: &str,
    api: &str,
    sparql: &str,
    claims: &[String],
) -> Result<Vec<String>, anyhow::Error> {
    use itertools::Itertools;
    if claims.is_empty() {
        return Err(anyhow::anyhow!("no claims provided, cannot find anything"));
    }
    let client = Client::new(api, sparql, topo_id_id)?;

    let claims = parse_claims(claims)?;
    let where_clause = format!(
        "?item {claims}.",
        claims = claims
            .iter()
            .map(|(p, v)| {
                if p.contains(':') {
                    // if the property contains a ':', we consider that we do not need to namespace it
                    // it makes it possible to look for exemple by label: rdfs:label
                    format!("{} {}", p, v)
                } else {
                    format!("wdt:{} {}", p, v)
                }
            })
            .join("; ")
    );

    let res = client.sparql.sparql(&["?item"], &where_clause)?;

    Ok(res
        .into_iter()
        .filter_map(|mut r| r.remove("item"))
        .filter_map(|u| transit_topo::sparql_client::read_id_from_url(&u))
        .collect())
}

fn main() {
    // by default the logs are not activated, if you want some, provide RUST_LOG=<level>
    // the logs are not activated since we want to use the stdout to pipe the results
    pretty_env_logger::init();

    let opt = Opt::from_args();

    match opt {
        Opt::Search {
            topo_id_id,
            api,
            sparql,
            claims,
        } => {
            let ids = search(&topo_id_id, &api, &sparql, &claims).expect("impossible to search:");
            for id in ids {
                println!("{}", id);
            }
        }
    }
}
