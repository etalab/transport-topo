use itertools::Itertools;
use regex::Regex;
use structopt::StructOpt;
use transit_topo::{
    clients::{api_client, sparql_client::read_id_from_url},
    topo_query::TopoQuery,
    GtfsImporter,
};

use clap::arg_enum;

arg_enum! {
    #[derive(Debug)]
    enum EntityType {
        Item,
        StringProperty,
        ItemProperty,
        UrlProperty,
    }
}

impl EntityType {
    fn get_object_type(&self) -> api_client::ObjectType {
        match self {
            EntityType::Item => api_client::ObjectType::Item,
            EntityType::StringProperty => {
                api_client::ObjectType::Property(api_client::PropertyDataType::String)
            }
            EntityType::ItemProperty => {
                api_client::ObjectType::Property(api_client::PropertyDataType::Item)
            }
            EntityType::UrlProperty => {
                api_client::ObjectType::Property(api_client::PropertyDataType::Url)
            }
        }
    }
}

lazy_static::lazy_static! {
    static ref CLAIM_REGEX: Regex = Regex::new(r"^(.*)=(.*)$").unwrap();

    static ref CLAIM_ITEM_REGEX: Regex = Regex::new(r"^wd:(.*)$").unwrap();
}

#[derive(StructOpt, Debug)]
#[structopt(name = "entities")]
enum Opt {
    Search {
        /// Identifier of the topo id property
        #[structopt(long, default_value = "P1")]
        topo_id_id: String,

        /// Endpoint of the wikibase api
        #[structopt(short = "a", long = "api")]
        _api: Option<String>,

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
    Create {
        /// Identifier of the topo id property
        #[structopt(long, default_value = "P1")]
        topo_id_id: String,

        /// Endpoint of the wikibase api
        #[structopt(short, long, default_value = "http://localhost:8181/api.php")]
        api: String,

        /// Endpoint of the sparql query service
        #[structopt(short, long, default_value = "http://localhost:8989/bigdata/sparql")]
        sparql: String,

        /// type of the entity
        #[structopt(short = "t", long = "type",
                    possible_values = &EntityType::variants(), case_insensitive = true)]
        entity_type: EntityType,

        /// Label of the new entity.
        label: String,

        /// Extra claim with the form P42=foobar. Can be repeated
        /// Those claims are used to check the unicity of the entity.
        /// known entities can be used in the form  `@<known_entity>`
        /// `known_entity can be the name of the fields in known_entities::Properties or known_entities::Items
        /// for example to add a claims saying that the entity should be a `instance of` `producer`:
        /// --claim "@instance_of=@producer"
        #[structopt(short, long = "unique-claim")]
        unique_claims: Vec<String>,

        /// Extra claim with the form P42:foobar. Can be repeated
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
    let claims: Result<Vec<(String, String)>, anyhow::Error> = claims
        .iter()
        .map(|claim| {
            let captures = CLAIM_REGEX
                .captures(&claim)
                .ok_or_else(|| anyhow::anyhow!("Could not parse claim {}", claim))?;
            Ok((captures[1].to_owned(), captures[2].to_owned()))
        })
        .collect();

    Ok(replace_known_entities(claims?, entities))
}

fn search(topo_id_id: &str, sparql: &str, claims: &[String]) -> Result<Vec<String>, anyhow::Error> {
    if claims.is_empty() {
        return Err(anyhow::anyhow!("no claims provided, cannot find anything"));
    }
    let query = TopoQuery::new(sparql, topo_id_id)?;

    let claims = parse_claims(claims, &query.known_entities)?;

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

    let res = query.client.sparql(&["?item"], &where_clause)?;

    Ok(res
        .into_iter()
        .filter_map(|mut r| r.remove("item"))
        .filter_map(|u| read_id_from_url(&u))
        .collect())
}

fn create_entity(
    entity_type: EntityType,
    label: &str,
    topo_id_id: &str,
    api: &str,
    sparql: &str,
    unique_claims: &[String],
    claims: &[String],
) -> Result<String, anyhow::Error> {
    let importer = GtfsImporter::new(api, sparql, topo_id_id)?;

    let parsed_unique_claims = parse_claims(unique_claims, &importer.query.known_entities)?;

    let where_clause = format!(
        r#"?item rdfs:label "{}"@en; {}."#,
        label,
        parsed_unique_claims
            .iter()
            .map(|(p, v)| format!("wdt:{} {}", p, v))
            .join("; ")
    );
    // We check that there is not yet an entity with this label
    match importer
        .query
        .client
        .sparql(&["?item"], &where_clause)?
        .into_iter()
        .filter_map(|r| read_id_from_url(&r["item"]))
        .collect::<Vec<_>>()
        .as_slice()
    {
        [id] => {
            log::info!(
                "entity {}, with unique_claims {:?} already exists with id {}",
                label,
                &unique_claims,
                id
            );
            Ok(id.to_owned())
        }
        [] => {
            log::info!("no entity \"{}\" exists, creating one", label);
            let claims: Vec<_> = parse_claims(claims, &importer.query.known_entities)?
                .iter()
                .chain(parsed_unique_claims.iter())
                .map(|(prop, value)| {
                    //it's kind of a hack, but the sparql api need namespace (`wdt:` for properties and `wd:` for items)
                    // for the rest api does not want those namespace
                    // so we use those namespace to know if the claim is on a item or a string
                    let prop = prop.replace("wdt:", "");
                    // same, the api does not want '<>' around the urls (but the sparql does)
                    let value = value.replace("<", "");
                    let value = value.replace(">", "");

                    match CLAIM_ITEM_REGEX.captures(&value) {
                        None => api_client::claim_string(&prop, &value),
                        Some(c) => api_client::claim_item(&prop, &c[1]),
                    }
                })
                .collect();

            log::debug!("creating entity \"{}\" with claims {:?}", label, &claims);
            let id = importer.writer.client.create_object(
                entity_type.get_object_type(),
                label,
                claims,
            )?;
            log::info!("created entity \"{}\" with id {}", label, id);
            Ok(id.to_owned())
        }
        l => {
            log::info!(
                "too many entities with label {} and unique claims {:?}: {:?}",
                label,
                &unique_claims,
                l
            );
            Err(anyhow::anyhow!("too many entities"))
        }
    }
}

fn main() {
    // by default the logs are not activated, if you want some, provide RUST_LOG=<level>
    // the logs are not activated since we want to use the stdout to pipe the results
    pretty_env_logger::init();

    let opt = Opt::from_args();

    match opt {
        Opt::Search {
            topo_id_id,
            _api,
            sparql,
            claims,
        } => {
            let ids = search(&topo_id_id, &sparql, &claims).expect("impossible to search:");
            for id in ids {
                println!("{}", id);
            }
        }
        Opt::Create {
            topo_id_id,
            api,
            sparql,
            entity_type,
            label,
            unique_claims,
            claims,
        } => {
            let id = create_entity(
                entity_type,
                &label,
                &topo_id_id,
                &api,
                &sparql,
                &unique_claims,
                &claims,
            )
            .expect("impossible to create entity");
            println!("{}", id);
        }
    }
}
