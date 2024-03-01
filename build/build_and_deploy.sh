#!/bin/sh

# -- start first time only run

# sudo useradd -s /sbin/nologin homecontroluser
# sudo usermod -a -G users homecontroluser

# make directory for service to live in
# sudo mkdir /var/homecontrolwebsvc

# -- end first time only run

$cur_dir=$(dirname $0)

systemctl stop homecontrol

pushd ..
cargo build
sudo cp ./build/homecontrol.service /etc/systemd/system/homecontrol.service
sudo cp ./target/debug/hc-rs /var/homecontrol/hc-rs
popd

sudo systemctl start homecontrol
sudo systemctl status homecontrol



