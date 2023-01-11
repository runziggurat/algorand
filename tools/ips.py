# Tools for creating ability to test nodes with different source IP using either virtual linux dummy devices
# or plain devices that can be addressed with multiple addresses (works on Linux and Mac).
# Moreover, it generates list of valid addresses (where ip/ipconfig command invocation completed successfully) and
# writes that list to the Rust file.
# When testing is done, just change --op to 'remove' and run the script again to remove all the dummy devices (or aliases).
#
# Linux and Mac supported! User must be in sudoers file to use this script.
#
# User can tweak settings using command line.
# Just run python ips.py --help for more information about parameters.
# Either --dev_prefix or --dev should be specified.
# Sample invocation:
# add whole 9.1.1.0/24 subnet creating dummy devices test_zeth0...test_zeth248 and resulting IPs to file: src/tools/ips.rs
# python3 ./tools/ips.py --subnet 9.1.1.0/24 --file src/tools/ips.rs --dev_prefix test_zeth
#
# remove all test_zeth* devices from 24 subnet (clear src/tools/ips.rs file)
# python3 ./tools/ips.py --subnet 9.1.1.0/24 --file src/tools/ips.rs --dev_prefix test_zeth --op remove
#
# add whole 9.1.1.0/24 subnet to device lo (for Mac use lo0) and resulting IPs to file: src/tools/ips.rs
# python3 ./tools/ips.py --subnet 9.1.1.0/24 --file src/tools/ips.rs --dev lo
#
# remove whole 9.1.1.0/24 subnet from device lo (clear src/tools/ips.rs file)
# python3 ./tools/ips.py --subnet 9.1.1.0/24 --file src/tools/ips.rs --dev lo --op remove

import argparse
import ipaddress
import os
import random
import sys

# The first sudo command will not be executed in the background, which will allow the user to enter the sudo credentials.
run_in_bg = ''

def generate_hwaddr():
    return "02:00:00:%02x:%02x:%02x" % (random.randint(0, 255),
                                        random.randint(0, 255),
                                        random.randint(0, 255))


def generate_dev(device_name, ip_addr):
    cmd = 'sudo ip link add ' + device_name + ' type dummy && '
    cmd += 'ifconfig ' + device_name + ' hw ether ' + generate_hwaddr() + ' && '
    cmd += 'ip addr add ' + ip_addr + ' dev ' + device_name + ' && '
    cmd += 'ip link set ' + device_name + ' up'
    print(cmd)
    return os.system(cmd)


def remove_dev(device_name, ip_addr):
    cmd = 'sudo ip link delete ' + device_name
    print(cmd)
    return os.system(cmd)


def add_addr_to_existing_dev(device_name, ip_addr):
    global run_in_bg

    if sys.platform.startswith('linux'):
        cmd = 'sudo ip addr add ' + ip_addr + ' dev ' + device_name + ' && '
        cmd += 'ip link set ' + device_name + ' up'
    else:
        cmd = 'sudo ifconfig ' + device_name + ' alias ' + ip_addr + '/32 ' + run_in_bg
    print(cmd)

    run_in_bg = '&'
    return os.system(cmd)


def delete_addr_from_existing_dev(device_name, ip_addr):
    if sys.platform.startswith('linux'):
        cmd = 'sudo ip addr del ' + ip_addr + '/32 dev ' + device_name
    else:
        cmd = 'sudo ifconfig ' + device_name + ' -alias ' + ip_addr
    print(cmd)
    return os.system(cmd)


parser = argparse.ArgumentParser(description='Setting interfaces and generating Rust file with IPs.')
parser.add_argument('--subnet', nargs='?', default='1.1.1.0/28', help='Subnet to generate', type=str)
parser.add_argument('--file', nargs='?', default='ips.rs', help='Output file with IPs (default ips.rs)', type=str)
parser.add_argument('--op', nargs='?', default='add', help='add/remove operation', type=str)
parser.add_argument('--dev_prefix', nargs='?', help='How to prefix dummy devs (Linux only)(def: test_dummyX)', type=str)
parser.add_argument('--dev', nargs='?', help='Device to add addresses (Linux and Mac)', type=str)

args = parser.parse_args()

subnet = args.subnet
file = args.file
op = args.op

if not sys.platform.startswith('linux') and not sys.platform.lower().startswith('darwin'):
    print('Script work only under Linux or MacOS')
    sys.exit(0)

if args.dev_prefix is None and args.dev is None:
    print('Either --dev_prefix or --dev must be specified')
    sys.exit(0)

if op not in ['add', 'remove']:
    print('Operation must be either add or remove')
    sys.exit(0)

dev = None
dev_prefix = None

if op == 'add':
    operation_dev = 'generate_dev'
    operation_addr = 'add_addr_to_existing_dev'
else:
    operation_dev = 'remove_dev'
    operation_addr = 'delete_addr_from_existing_dev'

if args.dev is not None:
    dev = args.dev

if sys.platform.startswith('linux'):
    if args.dev_prefix is not None :
        dev_prefix = args.dev_prefix
    else:
        dev_prefix = 'test_dummy'

ips = []

for ip in ipaddress.IPv4Network(subnet):
    ips.append(ip)

i = 0

with open(file, 'w') as f:
    f.write('pub const IPS: & [& str] = &[ \n')
    for ip in ips:
        if dev is not None:
            res = eval(operation_addr)(dev, str(ip))
        else:
            res = eval(operation_dev)(dev_prefix + str(i), str(ip))
        if res == 0 and op == 'add':
            f.write('\t"' + str(ip) + '",\n')
        i += 1

    f.write('];\n\n')
