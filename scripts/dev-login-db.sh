#!/bin/bash

set -e

DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)

cd $DIR

function get_config_value() {
    local val=$(sed -ne "s/^\s*$1.*\"\(.*\)\"/\1/p" Config.toml |  sed -n "$2p")
    echo $val
}

if [ ! -f Config.toml ]; then
    echo "Config.toml not found" && exit 1
fi

PG_HOST="$(get_config_value host 1)"
PG_HOSTNAME="$(echo $PG_HOST | cut -d: -f1)"
PG_PORT="$(echo $PG_HOST | cut -d: -f2)"
PG_USERNAME="$(get_config_value username 1)"
PG_PASSWORD="$(get_config_value password 1)"
PG_DB="$(get_config_value db 1)"

echo postgres://$PG_USERNAME:$PG_PASSWORD@$PG_HOSTNAME:$PG_PORT/$PG_DB;
psql postgres://$PG_USERNAME:$PG_PASSWORD@$PG_HOSTNAME:$PG_PORT/$PG_DB;
