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

Let's dive into the config file of node one.

```toml
[log]
type = "console"
level = "debug"

[discovery]
type = "constant"
addresses = [ ]

[cipher]
type = "symmetric"
key = "000102030405060708090a0b0c0d0e0f"

[transport]
type = "direct"
local_address = "127.0.0.1:4001"
request_timeout_ms = 5000
balancer = { type = "dynamic_round_robin" }

[[relay]]
type = "http_static"
address = "127.0.0.1:4200"
header_field = "X-Delix-Service"

[[relay.service]]
name = "slashdot"
address = "216.34.181.45:80"
```

The `discovery` section contains the field `addresses` which holds a list of IPs (with ports) that is used during
the node's boot up to search for other nodes. Since node `one` is the first, the list is empty here.

In the `cipher` section is the `key` defined for the encryption and authentication of the traffic between nodes.
The key can be 16, 24 or 32 bytes (hex encoded) long and will issue a AES-{128, 192 or 256}-GCM encryption. All nodes
in the network must share the same key.

In order to bind a node to an interface, `local_address` in the `transport` section must be set. If the interface
differs from the interface visible to other nodes, the field `public_address` can be set.

The `relay` section at the end, defines here a `http_static` relay that opens a port at `address` that takes HTTP
requests. The `header_field` in the request tells delix to which service the request should be routed to. The services
are defined in the `relay.service` section and define a name and an address which defines the endpoint where the
request is send. In this example it's the slashdot server.

## License

The code is licensed under [Apache 2.0](http://www.apache.org/licenses).
