mod utils;
use maplit::btreeset;
use pretty_assertions::assert_eq;

fn check_initiale_state(wikibase: &utils::Wikibase) {
    // we first check that our exists method cannot find a unknown object
    assert!(!wikibase.exists("pouet"));

    // then we check the real objects
    assert!(wikibase.exists("Instance of"));
    assert!(wikibase.exists("GTFS short name"));
    assert!(wikibase.exists("GTFS long name"));
    assert!(wikibase.exists("GTFS id"));
    assert!(wikibase.exists("Topo tools id"));

    assert!(wikibase.exists("Physical mode"));
    assert!(wikibase.exists("Producer"));
    assert!(wikibase.exists("Route"));
    assert!(wikibase.exists("Bus"));

    // we check all the objects with a topo_id
    assert_eq!(
        wikibase.get_topo_objects(),
        btreeset![
            "bus".to_owned(),
            "cable_car".to_owned(),
            "connecting_line".to_owned(),
            "data_source".to_owned(),
            "ferry".to_owned(),
            "file_format".to_owned(),
            "first_seen_in".to_owned(),
            "funicular".to_owned(),
            "gondola".to_owned(),
            "gtfs_id".to_owned(),
            "gtfs_long_name".to_owned(),
            "gtfs_name".to_owned(),
            "gtfs_short_name".to_owned(),
            "has_physical_mode".to_owned(),
            "instance_of".to_owned(),
            "part_of".to_owned(),
            "physical_mode".to_owned(),
            "produced_by".to_owned(),
            "producer".to_owned(),
            "railway".to_owned(),
            "route".to_owned(),
            "sha_256".to_owned(),
            "source".to_owned(),
            "stop_area".to_owned(),
            "stop_boarding_area".to_owned(),
            "stop_entrance".to_owned(),
            "stop_generic_node".to_owned(),
            "stop_point".to_owned(),
            "subway".to_owned(),
            "tool_version".to_owned(),
            "tramway".to_owned(),
        ],
    );
}

fn import_gtfs(docker: &utils::DockerContainerWrapper, producer_id: &str) {
    utils::run(
        "import-gtfs",
        &[
            "--producer",
            producer_id,
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
}

fn create_producer(
    label: &str,
    wikibase: &utils::Wikibase,
    docker: &utils::DockerContainerWrapper,
) -> String {
    utils::run(
        "entities",
        &[
            "create",
            label,
            "--type",
            "item",
            "--unique-claim",
            "@instance_of=@producer",
            "--api",
            &docker.api_endpoint,
            "--sparql",
            &docker.sparql_endpoint,
        ],
    );

    // we then query the base to find the id of the newly inserted producer
    wikibase
        .get_entity_id(label)
        .expect("impossible to find producer")
}

fn test_create_custom_property(wikibase: &utils::Wikibase, docker: &utils::DockerContainerWrapper) {
    let label = "Custom property";
    // we test the creation of a custom property using the `entity` cli tool
    let add_entity = || {
        utils::run(
            "entities",
            &[
                "create",
                label,
                "--type",
                "urlproperty",
                "--api",
                &docker.api_endpoint,
                "--sparql",
                &docker.sparql_endpoint,
            ],
        )
    };

    add_entity();

    // we then query the base to find the id of the newly inserted producer
    let prop_id = wikibase
        .get_entity_id(label)
        .expect("impossible to find property");

    // if we add it again, we find the same thing
    add_entity();
    assert_eq!(
        wikibase
            .get_entity_id(label)
            .expect("impossible to find property"),
        prop_id
    );
}

#[test]
fn simple_test() {
    let docker = utils::DockerContainerWrapper::new();

    utils::run(
        "prepopulate",
        &[
            "--api",
            &docker.api_endpoint,
            "--sparql",
            &docker.sparql_endpoint,
        ],
    );

    let wikibase = utils::Wikibase::new(&docker);
    check_initiale_state(&wikibase);

    // We call again the prepopulate, there shouldn't be any differences
    // since it should be idempotent
    utils::run(
        "prepopulate",
        &[
            "--api",
            &docker.api_endpoint,
            "--sparql",
            &docker.sparql_endpoint,
        ],
    );
    check_initiale_state(&wikibase);

    // we then need to add a producer
    let producer_id = create_producer("bob the bus mapper", &wikibase, &docker);

    // if we try to recreate the same producer, we should get the id of the old one
    assert_eq!(
        producer_id,
        create_producer("bob the bus mapper", &wikibase, &docker)
    );

    // we now import a gtfs
    import_gtfs(&docker, &producer_id);

    // there are 1 data sources with routes imported
    let data_sources = wikibase.get_producer_datasources_id(&producer_id);
    assert_eq!(data_sources.len(), 1);

    let data_source_id = data_sources.iter().next().unwrap();

    let data_source = wikibase.get_entity(data_source_id);

    assert!(data_source
        .label
        .starts_with(&format!("Data source for {} - imported ", &producer_id)));
    assert!(
        data_source.properties[&wikibase.properties().source].ends_with("tests/fixtures/gtfs.zip")
    );
    assert!(!data_source.properties[&wikibase.properties().sha_256].is_empty());
    assert!(!data_source.properties[&wikibase.properties().tool_version].is_empty());

    let all_objects = wikibase.get_all_items_for_datasource(data_source_id);
    assert_eq!(all_objects.len(), 14);

    let find_by_gtfs_id = |gtfs_id: &str| {
        all_objects
            .iter()
            .find(|o| o.gtfs_id == Some(gtfs_id.to_owned()))
    };

    let ab = find_by_gtfs_id("AB").expect(&format!("impossible to find AB"));
    assert_eq!(
        ab.label,
        "Bus Airport - Bullfrog (bob the bus mapper)".to_owned()
    );
    assert_eq!(ab.instance_of, "Route".to_owned());

    let instance_of = |id| {
        &find_by_gtfs_id(id)
            .expect("impossible to find obj")
            .instance_of
    };

    for route in &["BFC", "STBA", "CITY", "AAMV"] {
        assert_eq!(instance_of(route), "Route");
    }

    for stop in &["NADAV", "NANAA", "DADAN", "EMSI", "AMV"] {
        assert_eq!(instance_of(stop), "Stop point");
    }
    assert_eq!(instance_of("FUR_CREEK_RES"), "Stop area");
    assert_eq!(instance_of("BEATTY_AIRPORT"), "Stop entrance");
    assert_eq!(instance_of("BULLFROG"), "Stop generic node");
    assert_eq!(instance_of("STAGECOACH"), "Stop boarding area");

    // we reimport the gtfs
    import_gtfs(&docker, &producer_id);

    // there are now 2 datasources, because we do no merge.
    // It might change in the futur
    let new_datasources = wikibase.get_producer_datasources_id(&producer_id);
    assert_eq!(new_datasources.len(), 2);

    let new_datasource: std::collections::BTreeSet<_> =
        new_datasources.difference(&data_sources).collect();
    assert_eq!(new_datasource.len(), 1);

    let all_objects = wikibase.get_all_items_for_datasource(new_datasource.iter().next().unwrap());
    assert_eq!(all_objects.len(), 0);

    let ab = find_by_gtfs_id("AB").expect(&format!("impossible to find AB"));
    assert_eq!(
        ab.label,
        "Bus Airport - Bullfrog (bob the bus mapper)".to_owned()
    );
    assert_eq!(ab.instance_of, "Route".to_owned());

    // check that giving an invalid producer id does not work
    assert!(!utils::unchecked_run(
        "import-gtfs",
        &[
            "--producer",
            "Q12345", // this id does not exists
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
    )
    .success());

    // same with a valid id, but not a producer
    assert!(!utils::unchecked_run(
        "import-gtfs",
        &[
            "--producer",
            &wikibase.items().route, // 'route' exists in wikibase (add by the prepopulate), but it is not a producer
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
    )
    .success());

    let bullfrog = find_by_gtfs_id("BULLFROG").expect(&format!("impossible to find BULLFROG"));
    let fur_creek_res =
        find_by_gtfs_id("FUR_CREEK_RES").expect(&format!("impossible to find FUR_CREEK_RES"));
    let bullfrog_details = wikibase.get_entity(&bullfrog.id);
    println!("props: {:#?}", bullfrog_details.properties);
    assert_eq!(
        bullfrog_details.properties[&wikibase.properties().part_of].value(),
        fur_creek_res.id
    );

    test_create_custom_property(&wikibase, &docker);
}
