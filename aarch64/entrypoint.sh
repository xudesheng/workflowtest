#!/bin/sh -l
set -eux

# attention: this script is executed as root and you need to make sure it's a+x before you build the image
# Once inside your Rust project's root where you would normally cargo deb..
#docker run -v $PWD:/volume --rm --env GITHUB_WORKSPACE=. -t namespace/tag cargo deb

# Taken from https://github.com/zhxiaogg/cargo-static-build, unsure if all needed or not

# hack, move home to $HOME(/github/home)
ln -s /root/.cargo $HOME/.cargo
ln -s /root/.rustup $HOME/.rustup

# go to the repo root
cd $GITHUB_WORKSPACE
sh -c "$*"
chmod 0777 ./target