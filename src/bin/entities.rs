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

        /// Extra claim with the form P42=foobar. Can be repeated
        /// known entities can be used in the form  `@<known_entity>`
        /// `known_entity can be the name of the fields in known_entities::Properties or known_entities::Items
        /// for example to add a claims saying that the entity should be a `instance of` `producer`:
        /// --claim "@instance_of=@producer"
        #[structopt(short, long = "claim")]
        claims: Vec<String>,
    },
}

// convert the Json representation of a simple struct (either client::Properties or client::Items)
// into a hashmap field => value
// it has lots of `expect`, because it should never fail, as it depends on the code (so checked at build time)
fn as_map(val: serde_json::Value) -> std::collections::HashMap<String, String> {
    val.as_object()
        .expect("invalid value")
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().expect("value not string").to_owned()))
        .collect()
}

// replace in the claims known properties or known items
// the known fields are taken as "@field"
// for example to add a claims saying that the entity should be a `instance of` `producer`:
// --claims "@instance_of=@producer"
fn replace_known_entities(
    claims: Vec<(String, String)>,
    entities: &transit_topo::known_entities::EntitiesId,
) -> Vec<(String, String)> {
    let prop =
        as_map(serde_json::to_value(&entities.properties).expect("impossible to serialize prop"));
    let items =
        as_map(serde_json::to_value(&entities.items).expect("impossible to serialize items"));

    claims
        .into_iter()
        .map(|(mut claim_prop, mut claim_value)| {
            for (k, v) in prop.iter() {
                claim_prop = claim_prop.replace(&format!("@{}", k), v);
            }
            for (k, v) in items.iter() {
                claim_value = claim_value.replace(&format!("@{}", k), &format!("wd:{}", v));
            }
            (claim_prop, claim_value)
        })
        .collect()
}

fn parse_claims(
    claims: &[String],
    entities: &transit_topo::known_entities::EntitiesId,
) -> Result<Vec<(String, String)>, anyhow::Error> {
    let re = regex::Regex::new(r"^(.*)=(.*)$")?;

    let claims: Result<Vec<(String, String)>, anyhow::Error> = claims
        .iter()
        .map(|claim| {
            let captures = re
                .captures(&claim)
                .ok_or_else(|| anyhow::anyhow!("Could not parse claim {}", claim))?;
            Ok((captures[1].to_owned(), captures[2].to_owned()))
        })
        .collect();

    Ok(replace_known_entities(claims?, entities))
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

    let claims = parse_claims(claims, &client.sparql.known_entities)?;

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
