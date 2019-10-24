# Transit topo tools

https://topo.transport.data.gouv.fr/ a semantic database of transit objects (stops, lines, networks…).

This program is a toolkit to populate that database from a [GTFS](https://gtfs.org) file.

It allows to populate missing features in the database.

The tool is designed to be idempotent: importing twice the same file, or different files from the same producer won’t generate any duplicate.

## Installation

Transit topo tools are written in [Rust](https://www.rust-lang.org/).

You need an up to date rust tool-chain (commonly installed with [rustup](https://rustup.rs/)).

## Configuration

You need to configure `config.toml` in order to set which are the ids of basic identifiers (“bus”…) or property (“gtfs_id”).

See `config.example.toml` how to set it up.

## Running

Identifiers of entities can be the same across different producers. That is why we require to set a producer with the `--producer` (or `-p`) flag.

For instance, to import all the lines of the producer `Q4` from the local file `gtfs.zip`, run:

    cargo run --release --bin import_lines -- -p Q4 -i gtfs.zip

## Testing

The integration tests are based on [docker](https://www.docker.com) and [docker-compose](https://docs.docker.com/compose/), you need those tools installed.

To run the tests run:

    cargo test

Note: docker need some root privileges, you might need to run this with more privileges (or [use other controversial means](https://docs.docker.com/install/linux/linux-postinstall/))
