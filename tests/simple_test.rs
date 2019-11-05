mod utils;
use maplit::hashset;
use transit_topo::api_client::{ObjectType, PropertyDataType};

fn check_initiale_state(wikibase: &utils::Wikibase) {
    // we first check that our exists method cannot find a unknown object
    assert!(!wikibase.exists(ObjectType::Item, "pouet"));

    // then we check the real objects
    assert!(wikibase.exists(
        ObjectType::Property(PropertyDataType::String),
        "instance of"
    ));
    assert!(wikibase.exists(
        ObjectType::Property(PropertyDataType::String),
        "gtfs short name"
    ));
    assert!(wikibase.exists(
        ObjectType::Property(PropertyDataType::String),
        "gtfs long name"
    ));
    assert!(wikibase.exists(ObjectType::Property(PropertyDataType::String), "gtfs id"));
    assert!(wikibase.exists(
        ObjectType::Property(PropertyDataType::Item),
        "Topo tools id"
    ));

    assert!(wikibase.exists(ObjectType::Item, "physical mode"));
    assert!(wikibase.exists(ObjectType::Item, "producer"));
    assert!(wikibase.exists(ObjectType::Item, "route"));
    assert!(wikibase.exists(ObjectType::Item, "bus"));
    assert!(wikibase.exists(ObjectType::Item, "bob the bus mapper"));

    // we check all the objects with a topo_id
    assert_eq!(
        wikibase.get_topo_objects(),
        hashset![
            "producer".to_owned(),
            "line".to_owned(),
            "bob_the_bus_mapper".to_owned(),
            "instance_of".to_owned(),
            "physical_mode".to_owned(),
            "gtfs_short_name".to_owned(),
            "gtfs_long_name".to_owned(),
            "gtfs_id".to_owned(),
            "produced_by".to_owned(),
            "physical_mode".to_owned(),
            "has_physical_mode".to_owned(),
            "tramway".to_owned(),
            "subway".to_owned(),
            "railway".to_owned(),
            "bus".to_owned(),
            "ferry".to_owned(),
            "cable_car".to_owned(),
            "gondola".to_owned(),
            "funicular".to_owned(),
        ]
    );
}

#[test]
fn simple_test() {
    let docker = utils::DockerContainerWrapper::new();

    utils::run("prepopulate", &["--api", &docker.api_endpoint]);

    let wikibase = utils::Wikibase::new(&docker);
    check_initiale_state(&wikibase);

    // We call again the prepopulate, there shouldn't be any differences
    // since it should be idempotent
    utils::run("prepopulate", &["--api", &docker.api_endpoint]);
    check_initiale_state(&wikibase);

    // we now import a gtfs
    utils::run(
        "import-gtfs",
        &[
            "--producer",
            "Q4",
            "--input-gtfs",
            &format!(
                "{}/tests/fixtures/gtfs.zip",
                std::env::var("CARGO_MANIFEST_DIR").expect("impossible to find env var")
            ),
            "--api",
            &docker.api_endpoint,
            "--sparql",
            &docker.sparql_endpoint,
        ],
    );

    // there are 5 routes imported
    assert_eq!(wikibase.count_by_producer("Q4"), 5);
    assert_eq!(
        wikibase.gtfs_id_by_producer("Q4"),
        hashset! {
            "AB".to_owned(),
            "BFC".to_owned(),
            "STBA".to_owned(),
            "CITY".to_owned(),
            "AAMV".to_owned(),
        }
    );
}
