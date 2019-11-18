# Transit topo tools

https://topo.transport.data.gouv.fr/ a semantic database of transit objects (stops, lines, networks…).

This program is a toolkit to populate that database from a [GTFS](https://gtfs.org) file.

It allows to populate missing features in the database.

The tool is designed to be idempotent: importing twice the same file, or different files from the same producer won’t generate any duplicate.

## Installation

Transit topo tools are written in [Rust](https://www.rust-lang.org/).

You need an up to date rust tool-chain (commonly installed with [rustup](https://rustup.rs/)).

## Tools

Note: all binaries expose a `--help` cli argument to document all the available arguments.

#### GTFS import

You can use the tool `import-gtfs` to import a GTFS in TOPO.

Identifiers of entities can be the same across different producers. That is why we require to tell which `producer` is providing the GTFS.
The `producer` needs to be already added to the transport TOPO instance.

    cargo run --release --bin import-gtfs -- --api <url of the wikibase api> --sparql <url of the sparql api> --producer <id of the producer> -i <path to gtfs.zip>

#### Entity

You can use the tool `entities` to add or search for entity in TOPO.

This can be useful to explore or manage TOPO with cli tool.

##### Search

You can search for entities with some claims with the `search` endpoint.

Eg. to get the id of the item with the topo_id_id (`P1`) "route":

    cargo run --bin entities -- search --api <url of the wikibase api> --sparql <url of the sparql api> --claim 'P1="route"'

Note: the `--claim` is directly passed to the sparql endpoint, so you need to know a bit sparql to use this.
Note: The string must be quoted with `""`, the URL with `<>`


###### Examples uses

* query entities with the label "bob" (note the `""` around the label, and the `@en` telling where looking for the english label):

    cargo run --bin entities -- search --api <url of the wikibase api> --sparql <url of the sparql api> --claim 'rdfs:label="bob"@en'

* query all producers:

    cargo run --bin entities -- search --api <url of the wikibase api> --sparql <url of the sparql api> --claim '@instance_of=@producer'

* query entities that have property P42 with value `https://transport.data.gouv.fr/datasets/5bfd2e81634f4122b3023260`, which is of type `url` (note the `<>` around the url):

    cargo run --bin entities -- search --api <url of the wikibase api> --sparql <url of the sparql api> --claim 'P42=<https://transport.data.gouv.fr/datasets/5bfd2e81634f4122b3023260>'

##### Create

You can create entities with the `create` endpoint.

###### Examples uses

* create a property "data_gouv_id" of type url:

    cargo run --bin entities create "data_gouv_url" --type urlproperty

* To create an item "bob", which is an instance of `producer` (and we want only one producer named "bob"), with a property data_gouv_url "https://www.data.gouv.fr/datasets/5dc41db9634f417610c24a9d" (If the property does not exists yet, we create it) :

    cargo run --bin entities create "bob" --type item --unique-claim "@instance_of=@producer" --claim "$(cargo run --bin entities create "data_gouv_url" --type urlproperty)=<https://www.data.gouv.fr/datasets/5dc41db9634f417610c24a9d>"


## Contributing

### Building

To build the project, run:

    make build

### Testing

The integration tests are based on [docker](https://www.docker.com) and [docker-compose](https://docs.docker.com/compose/), you need those tools installed.

To run the tests run:

    make test

Note: docker need some root privileges, you might need to run this with more privileges (or [use other controversial means](https://docs.docker.com/install/linux/linux-postinstall/))

### Running locally

#### Set up
This project needs a running wikibase instance. For dev purpose, you can use the provided docker-compose.

To set up a wikibase instance, you can use the Makefile target:

    make docker-up

Note: the docker files are split between a minimal one (used in the integration tests) and another one used to ease use. So if you want to run custom `docker-compose` command, use:

    docker-compose -f tests/minimal-docker-compose.yml -f local-compose.yml <your-command>

The wikibase instance is quite long to start, you'll need to wait a bit (several minutes).
You know the services are available by querying the wikibase api:

    curl --head http://localhost:8181/api.php # This need to return a http response, with a `200` status code

When the service is available, you can prepopulate the base (to add the mandatory data, like the `instance of` property, ...)

    cargo run --release --bin prepopulate -- --api http://localhost:8181/api.php


#### Data producer

The idea is that each GTFS provider needs to have its own `producer` page in transit_topo.

This way all data added by this `producer` will be attached to it.

To create a producer, you can use the cli tool provided:

    cargo run --release --bin entities -- create <name of the producer> --type item --unique-claim @instance_of=@producer --api http://localhost:8181/api.php --sparql http://localhost:8989/bigdata/sparql

The cli tool will give you an ID. Note this id, it will be needed by the other cli tools.

Note: if you forgot the id, you can call again the cli tool, it will not recreate a producer with the same label.

#### Import GTFS

Once this is done, you can import the GTFS.
    
So to import the GTFS run:

    cargo run --release --bin import-gtfs -- --api http://localhost:8181/api.php --sparql http://localhost:8989/bigdata/sparql --producer <id of the producer> -i <path to gtfs.zip>
