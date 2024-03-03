#!/bin/bash

# -- start first time only run

# make directory for service to live in
# sudo mkdir -p /var/homecontrol/resources

# -- end first time only run

# get project root dir, which is the parent of the script dir
cur_dir=$(realpath $(dirname $0))
root_dir="$(dirname "$cur_dir")"
echo "$root_dir"

sudo systemctl stop homecontrol

cargo build
sudo cp $root_dir/build/homecontrol.service /etc/systemd/system/homecontrol.service
sudo cp $root_dir/target/debug/hc-rs /var/homecontrol/hc-rs
sudo cp $root_dir/resources/* /var/homecontrol/resources

sudo systemctl enable homecontrol
sudo systemctl start homecontrol
sudo systemctl --no-pager status homecontrol



