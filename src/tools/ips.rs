// If the IPS array is empty then it means that source IP addresses have not been
// generated and assigned to dummy devices. In such case tests should not bound to any
// specific address and use local pool instead.

/// Reference to a static array of IP addresses (represented as str).
/// If array is empty generate new addresses using eg.: for Linux:
/// sudo python3 ./tools/ips.py --subnet 1.1.1.0/24 --file src/tools/ips.rs --dev_prefix test_zeth
/// for MacOS:
/// sudo python3 ./tools/ips.py --subnet 1.1.1.0/24 --file src/tools/ips.rs --dev lo0
/// For more information read the documentation of the ips.py script.
pub const IPS: &[&str] = &[];
