# Ziggurat x Algorand (WIP)

> Work in progress

The Ziggurat implementation for Algorand's `algod` nodes.

## Getting started

1. Ensure you have `rust` including its package manager `cargo` installed. You can verify that `cargo` is on your path by running `cargo version`.
2. Clone this repository.
3. Build [go-algorand](https://github.com/algorand/go-algorand) from source.
4. Run the setup script:
```zsh
    tools/setup_env.sh
```

In case algorand files are installed in a specific location, export that location to the `ALGORAND_BIN_PATH
environment variable and rerun the setup script:
```zsh
    export ALGORAND_BIN_PATH="$HOME/node/"   # example path
    tools/setup_env.sh
```
5. Run tests with the following command:
```zsh
    cargo +stable t -- --test-threads=1
```
