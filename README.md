# Ziggurat x Algorand
[![dependency status](https://deps.rs/repo/github/runziggurat/algorand/status.svg)](https://deps.rs/repo/github/runziggurat/algorand)

The Ziggurat implementation for Algorand's `algod` nodes.

## Prerequisites

Ziggurat is written in stable Rust; you can install the Rust toolchain by following the official instructions [here](https://www.rust-lang.org/learn/get-started).

## Getting started

### Preconditions

1. Clone this repository.
2. Build [go-algorand](https://github.com/algorand/go-algorand) from source.
3. Run the setup script:
```zsh
 tools/setup_env.sh
```
In case algorand files are installed in a specific location, export that location to the `ALGORAND_BIN_PATH`
environment variable and rerun the setup script:
```zsh
 export ALGORAND_BIN_PATH="$HOME/node"   # example path
 tools/setup_env.sh
```

### Run tests
Run conformance and resistance tests with the following command:

```zsh
 cargo +stable test
```

### Run performance tests

Consult the [performance tests readme](PERF.md) for details on running these tests.


## Test Status

Short overview of test cases and their current status. In case of failure, the behaviour observed is usually documented in the test case.

These results were obtained by running the test suite against [Algorand v3.12.2-stable](https://github.com/algorand/go-algorand/releases/tag/v3.12.2-stable) (181490e3).

| Legend |               |
| :----: | ------------- |
|   ✓    | pass          |
|   ✖    | fail          |
|   -    | unimplemented |

### Conformance

|             Test Case             | Algod  | Additional Information                                                      |
| :-------------------------------: | :----: | :-------------------------------------------------------------------------- |
| [001](SPEC.md#ZG-CONFORMANCE-001) |   ✓    |                                                                             |
| [002](SPEC.md#ZG-CONFORMANCE-002) |   ✓    |                                                                             |
| [003](SPEC.md#ZG-CONFORMANCE-003) |   ✓    |                                                                             |
| [004](SPEC.md#ZG-CONFORMANCE-004) |   ✓    |                                                                             |
| [005](SPEC.md#ZG-CONFORMANCE-005) |   ✓    |                                                                             |
| [006](SPEC.md#ZG-CONFORMANCE-006) |   ✓    |                                                                             |
| [007](SPEC.md#ZG-CONFORMANCE-007) |   ✓    |                                                                             |
| [008](SPEC.md#ZG-CONFORMANCE-008) |   ✓    |                                                                             |
| [009](SPEC.md#ZG-CONFORMANCE-009) |   ✖    | The PingReply handler doesn't exist anymore in the go-algorand codebase     |
| [010](SPEC.md#ZG-CONFORMANCE-010) |  ✓/✖   | Only BlockAndCert request is supported, other type requests are unsupported |
| [011](SPEC.md#ZG-CONFORMANCE-011) |   ✓    |                                                                             |
| [012](SPEC.md#ZG-CONFORMANCE-012) |   ✓    |                                                                             |
| [013](SPEC.md#ZG-CONFORMANCE-013) |   ✓    |                                                                             |

### Performance

|             Test Case             | Algod  | Additional Information                                                      |
|:---------------------------------:| :----: | :-------------------------------------------------------------------------- |
| [001](SPEC.md#ZG-PERFORMANCE-001) |   ✓    |                                                                             |
| [002](SPEC.md#ZG-PERFORMANCE-002) |   ✓    |                                                                             |

### Resistance

|             Test Case             | Algod  | Additional Information                                                                     |
| :-------------------------------: | :----: | :----------------------------------------------------------------------------------------- |
| [001](SPEC.md#ZG-RESISTANCE-001)  |   ✖    | The node doesn't reject the connection in case a small amount of random data is sent       |
| [002](SPEC.md#ZG-RESISTANCE-002)  |  ✓/✖   | The procedure accepts sometimes invalid requests (should be improved)                      |
| [003](SPEC.md#ZG-RESISTANCE-003)  |   ✖    | The node doesn't reject the connection in most scenarios                                   |
| [004](SPEC.md#ZG-RESISTANCE-004)  |  ✓/✖   | The node won't reject the connection for enormously long and invalid messages              |
