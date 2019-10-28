mod utils;
use maplit::hashset;
use transitwiki::api_client::{ObjectType, PropertyDataType};

#[test]
fn simple_test() {
    let docker = utils::DockerContainerWrapper::new();

    utils::run(&docker, "prepopulate", &[]);

    let wikibase_client = utils::Wikibase::new(&docker);

    // we first check that our exists method cannot find a unknown object
    assert!(!wikibase_client.exists(ObjectType::Item, "pouet"));

    // then we check the real objects
    assert!(wikibase_client.exists(
        ObjectType::Property(PropertyDataType::String),
        "instance of"
    ));
    assert!(wikibase_client.exists(
        ObjectType::Property(PropertyDataType::String),
        "physical mode"
    ));
    assert!(wikibase_client.exists(
        ObjectType::Property(PropertyDataType::String),
        "gtfs short name"
    ));
    assert!(wikibase_client.exists(
        ObjectType::Property(PropertyDataType::String),
        "gtfs long name"
    ));
    assert!(wikibase_client.exists(ObjectType::Property(PropertyDataType::String), "gtfs id"));
    assert!(wikibase_client.exists(
        ObjectType::Property(PropertyDataType::Item),
        "Topo tools id"
    ));

    assert!(wikibase_client.exists(ObjectType::Item, "producer"));
    assert!(wikibase_client.exists(ObjectType::Item, "line"));
    assert!(wikibase_client.exists(ObjectType::Item, "bus"));
    assert!(wikibase_client.exists(ObjectType::Item, "bob the bus mapper"));

    // we check all the objects with a topo_id
    assert_eq!(
        wikibase_client.get_topo_objects(),
        hashset![
            "producer".to_owned(),
            "line".to_owned(),
            "bus".to_owned(),
            "bob_the_bus_mapper".to_owned(),
        ]
    );

    // We call again the prepopulate, there shouldn't be any differences
    // since it should be idempotent
    utils::run(&docker, "prepopulate", &[]);
    assert_eq!(
        wikibase_client.get_topo_objects(),
        hashset![
            "producer".to_owned(),
            "line".to_owned(),
            "bus".to_owned(),
            "bob_the_bus_mapper".to_owned(),
        ]
    );
}
