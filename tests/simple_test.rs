mod utils;
use maplit::hashset;
use transitwiki::api_client::{ObjectType, PropertyDataType};

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
        "physical mode"
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

    assert!(wikibase.exists(ObjectType::Item, "producer"));
    assert!(wikibase.exists(ObjectType::Item, "line"));
    assert!(wikibase.exists(ObjectType::Item, "bus"));
    assert!(wikibase.exists(ObjectType::Item, "bob the bus mapper"));

    // we check all the objects with a topo_id
    assert_eq!(
        wikibase.get_topo_objects(),
        hashset![
            "producer".to_owned(),
            "line".to_owned(),
            "bus".to_owned(),
            "bob_the_bus_mapper".to_owned(),
        ]
    );
}

#[test]
fn simple_test() {
    let docker = utils::DockerContainerWrapper::new();

    utils::run(&docker, "prepopulate", &[]);

    let wikibase = utils::Wikibase::new(&docker);
    check_initiale_state(&wikibase);

    // We call again the prepopulate, there shouldn't be any differences
    // since it should be idempotent
    utils::run(&docker, "prepopulate", &[]);
    check_initiale_state(&wikibase);

    // we now import a gtfs
    utils::run(
        &docker,
        "import_lines",
        &[
            "--producer",
            "Q4",
            "--input-gtfs",
            &format!(
                "{}/tests/fixtures/gtfs.zip",
                std::env::var("CARGO_MANIFEST_DIR").expect("impossible to find env var")
            ),
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
