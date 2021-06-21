#!/usr/bin/env bash
cd /liftinstall || exit 1

# setup NodeJS
curl -sL https://deb.nodesource.com/setup_12.x | bash -
# setup Yarn
curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add -
echo "deb https://dl.yarnpkg.com/debian/ stable main" > /etc/apt/sources.list.d/yarn.list

apt-get update
apt-get install -y libwebkit2gtk-4.0-dev libssl-dev nodejs yarn

yarn --cwd ui

cargo build
