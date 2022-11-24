#!/usr/bin/env bash
# This script sets up the environment for the Ziggurat test suite.
#
# The private network setup is explained here:
# [1] https://developer.algorand.org/docs/clis/goal/network/create/
#
# Telemetry config settings are explained here:
# [2] https://developer.algorand.org/docs/run-a-node/reference/telemetry-config/
#
# BaseLoggerDebugLevel is explained here:
# [3] https://developer.algorand.org/docs/run-a-node/reference/config/

set -e

# Algorand files
if [ -z $ALGORAND_BIN_PATH ]; then
    GOPATH=`go env GOPATH`
    ALGORAND_BIN_PATH="$GOPATH/bin"
fi
ALGOD_BIN_NAME="algod"
GOAL_CMD="$ALGORAND_BIN_PATH/goal"

# Ziggurat config files
ZIGGURAT_ALGORAND_DIR="$HOME/.ziggurat/algorand"
ZIGGURAT_ALGORAND_SETUP_DIR="$ZIGGURAT_ALGORAND_DIR/setup"
ZIGGURAT_ALGORAND_SETUP_CFG_FILE="$ZIGGURAT_ALGORAND_SETUP_DIR/config.toml"
# Private network
ZIGGURAT_ALGORAND_PN_DIR="$ZIGGURAT_ALGORAND_DIR/private_network"

setup_config_file() {
    echo "--- Setting up configuration file"
    echo "Creating $ZIGGURAT_ALGORAND_SETUP_CFG_FILE with contents:"
    mkdir -p $ZIGGURAT_ALGORAND_SETUP_DIR
    echo
    echo "# Algorand installation path" > $ZIGGURAT_ALGORAND_SETUP_CFG_FILE
    echo "path = \"$ALGORAND_BIN_PATH\"" >> $ZIGGURAT_ALGORAND_SETUP_CFG_FILE
    echo "# Start command with possible arguments" >> $ZIGGURAT_ALGORAND_SETUP_CFG_FILE
    echo "start_command = \"$ALGOD_BIN_NAME\"" >> $ZIGGURAT_ALGORAND_SETUP_CFG_FILE

    # Print file contents so the user can check whether the path is correct
    cat $ZIGGURAT_ALGORAND_SETUP_CFG_FILE
    echo
}

setup_private_network() {
    echo "--- Setting up private network files at the location $ZIGGURAT_ALGORAND_PN_DIR"
    $GOAL_CMD network create -r $ZIGGURAT_ALGORAND_PN_DIR -n private -t tools/ziggurat_network_template.json # see [1]
    echo

    # Copy telemetry config file manually to ensure nodes don't look for the global config file at ~/.algorand/
    cp tools/logging.config "$ZIGGURAT_ALGORAND_PN_DIR/Node0/"  # see [2]
    cp tools/logging.config "$ZIGGURAT_ALGORAND_PN_DIR/Node1/"  # see [2]

    update_config_file "$ZIGGURAT_ALGORAND_PN_DIR/Node0"
    update_config_file "$ZIGGURAT_ALGORAND_PN_DIR/Node1"
}

# Function appends attributes after the "Version" attribute in the configuration JSON file.
# The "Version" attribute must always be the first attribute specified in the JSON file.
#
# The input argument is the location of the node's directory.
update_config_file() {
    CFG_FILE="$1/config.json"

    # The default (unspecified) base log level is 4 (info). Use the most verbose log level 5 (debug / verbose) instead.
    echo "$(awk 'NR==3{print "\t\"BaseLoggerDebugLevel\": 5,"}1' $CFG_FILE)" > $CFG_FILE # see [3]

    # The default value is 30. The other workaround is using different 127.0.x.x addresses, but this is easier.
    echo "$(awk 'NR==3{print "\t\"MaxConnectionsPerIP\": 900,"}1' $CFG_FILE)" > $CFG_FILE # see [3]
}

# Verify the algod binary path using the version option
set +e; $ALGORAND_BIN_PATH/$ALGOD_BIN_NAME -v &> /dev/null; RET=$?; set -e;
if [ "$RET" != "0" ]; then
    echo "Aborting. Cannot find $ALGORAND_BIN_PATH/$ALGOD_BIN_NAME".
    exit 1
fi

# Verify the repo location
if [ "$(git rev-parse --is-inside-work-tree 2>/dev/null)" != "true" ]; then
    echo "Aborting. Use this script only from the ziggurat/algorand repo."
    exit 1
fi
REPO_ROOT=`git rev-parse --show-toplevel`
if [ "`basename $REPO_ROOT`" != "algorand" ]; then
    # Wrong root directory, check for rename compared to origin url.
    ORIGIN_URL=$(git config --local remote.origin.url|sed -n 's#.*/\([^.]*\)\.git#\1#p')
    if [ "$ORIGIN_URL" != "algorand" ]; then
        echo "Aborting. Use this script only from the ziggurat/algorand repo."
        exit 1
    fi
fi

# Setup the main ziggurat directory in the home directory
mkdir -p $ZIGGURAT_ALGORAND_DIR

# Change dir to ensure script paths are always correct
pushd . &> /dev/null
cd $REPO_ROOT;

setup_config_file
setup_private_network
echo "--- Setup successful"

popd &> /dev/null
