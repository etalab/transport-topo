mod utils;

#[test]
fn simple_test() {
    let docker = utils::DockerContainerWrapper::new();

    utils::run(&docker, "prepopulate", &[]);

    let wikibase_client = utils::Wikibase::new(&docker);

    // we first check that our exists method cannot find a unknown object
    assert!(!wikibase_client.exists("pouet"));

    // then we check the real objects
    assert!(wikibase_client.exists("producer"));
    // find properties too: "instance of"
    // find properties too: "physical mode"
    // find properties too: "gtfs short name"
    // find properties too: "gtfs long name"
    // find properties too: "gtfs id"
    assert!(wikibase_client.exists("line"));
    assert!(wikibase_client.exists("bus"));
    assert!(wikibase_client.exists("bob the bus mapper"));
}
