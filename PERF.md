## Run performance tests
Create a package of IP addresses (4k addresses) which are required for performance tests.

You must first fetch the `ips.py` script from the ziggurat-core repository.  Run this:

```bash
wget -O tools/ips.py https://raw.githubusercontent.com/runziggurat/ziggurat-core/main/ziggurat-core-scripts/ips.py
```


_NOTE: To run the `ips.py` script below, the user must be in sudoers file in order to use this script.
Script uses `ip`/`ipconfig` commands which require the sudo privilages._

From the root repository directory, depending on your OS, run one of the following commands.

#### Preconditions under Linux
Generate dummy devices with addresses:
```zsh
python3 ./tools/ips.py --subnet 1.1.0.0/20 --file tools/ips_list.json --dev_prefix test_zeth
```

#### Preconditions under MacOS
Add the whole subnet to the loopback device - can be also used on Linux (device name - Linux: `lo`, MacOS: `lo0`):
```zsh
python3 ./tools/ips.py --subnet 1.1.0.0/20 --file tools/ips_list.json --dev lo0
```
Increase the limit for the number of file descriptors:
```zsh
ulimit -n 65536
```

#### Run tests
Run performance tests with the following command:
```zsh
cargo +stable test performance --features performance
```
