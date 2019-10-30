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

#### Import GTFS
Once this is done, you can import the GTFS.
    
For dev purpose, a mock producer has been added by the `prepopulate`: `Q4`.

So to import the GTFS run:

    cargo run --release --bin import-gtfs -- --api http://localhost:8181/api.php --sparql http://localhost:8989/bigdata/sparql --producer Q4 -i <path to gtfs.zip>