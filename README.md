# Ziggurat x Algorand

*Note:* This project is a work in progress.

The Ziggurat implementation for Algorand's `algod` nodes.

## Prerequisites

Ziggurat is written in stable Rust; you can install the Rust toolchain by following the official instructions [here](https://www.rust-lang.org/learn/get-started).

## Getting started

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
4. Run tests with the following command:
```zsh
    cargo +stable t -- --test-threads=1
```

## Test Status

Short overview of test cases and their current status. In case of failure, the behaviour observed is usually documented in the test case.

These results were obtained by running the test suite against [Algorand v3.9.4-stable](https://github.com/algorand/go-algorand/releases/tag/v3.9.4-stable) (921e8f6f).

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
