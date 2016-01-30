# Delix

[![build status](https://secure.travis-ci.org/simia-tech/rust-delix.png)](http://travis-ci.org/simia-tech/rust-delix)

The idea behind delix is the create an overlay network that connects microservices. It uses semantic addressing
and takes care of encryption, fail-over and load balancing.

## Installation

    cargo install delix

## Example

Run three delix nodes in three different terminals.

    delix -c example/one.conf.toml
    delix -c example/two.conf.toml
    delix -c example/three.conf.toml

The node `one` opens an interface at `127.0.0.1:4200` which take http requests. All nodes have the service `slashdot`
configured. In order to request a response from the service run...

    curl -H 'Host: slashdot.org' -H 'X-Delix-Service: slashdot' http://127.0.0.1:4200

## License

The code is licensed under [Apache 2.0](http://www.apache.org/licenses).
