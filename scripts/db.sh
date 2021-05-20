#!/bin/bash

set -e

DIR=$(cd -P -- "$(dirname -- "$0")" && pwd -P)

GREEN="\033[1;32m"
RED="\033[0;31m"
END="\033[0m"
TOOLCHAIN="nightly"
DISTRIBUTION="$(lsb_release -is)"
DISTRIBUTION_VERSION="$(lsb_release -rs)"
PG_VERSION="10"

cd $DIR

if [ "$DISTRIBUTION" != "Ubuntu" ] && [ "$DISTRIBUTION" != "LinuxMint" ] && [  "$DISTRIBUTION" != "Linuxmint" ]; then
    echo -e "${RED}Error: distribution is not one of (ubuntu, linuxmint)${END}" && exit 1
fi

if [ "$DISTRIBUTION_VERSION" == "16.04" ]; then
    PG_VERSION="9.5"
elif [ "$DISTRIBUTION_VERSION" == "18.04" ]; then
    PG_VERSION="10"
elif [ "$DISTRIBUTION_VERSION" == "19.04" ]; then
    PG_VERSION="11"
elif [ "$DISTRIBUTION_VERSION" == "19.10" ]; then
    PG_VERSION="12"
elif [ "$DISTRIBUTION_VERSION" == "20.04" ] || [  "$DISTRIBUTION_VERSION" == "20" ]; then
    PG_VERSION="12"
fi

function get_config_value() {
    local val=$(sed -ne "s/^\s*$1.*\"\(.*\)\"/\1/p" Config.toml |  sed -n "$2p")
    echo $val
}

PG_HOST="$(get_config_value host 1)"
PG_USERNAME="$(get_config_value username 1)"
PG_PASSWORD="$(get_config_value password 1)"
PG_DB="$(get_config_value db 1)"
PG_HOSTNAME="$(echo $PG_HOST | cut -d: -f1)"
PG_PORT="$(echo $PG_HOST | cut -d: -f2)"

echo -e "${GREEN}=> Installing postgresql@${PG_VERSION}...${END}"
sudo apt install -y --no-install-recommends postgresql-$PG_VERSION libpq-dev build-essential

echo -e "${GREEN}=> Creating user $PG_USERNAME...${END}"
if [ -z "$(sudo -Hiu postgres psql -tAc "SELECT 1 FROM pg_roles WHERE rolname='$PG_USERNAME'")" ]; then
    sudo -Hiu postgres createuser --superuser $PG_USERNAME
fi
sudo -Hiu postgres psql -c "alter user $PG_USERNAME with password '$PG_PASSWORD';"

echo -e "${GREEN}=> Creating database $PG_DB...${END}"
if [ "$(sudo -Hiu postgres psql -lqt | cut -d"|" -f 1 | grep -cw "$PG_DB")" -eq 0 ]; then
    sudo -Hiu postgres createdb $PG_DB
fi
sudo -Hiu postgres psql -c "grant all privileges on database $PG_DB to $PG_USERNAME;"

echo -e "${GREEN}=> Configuring postgresql...${END}"
if [ "$(sudo cat /etc/postgresql/$PG_VERSION/main/pg_hba.conf | grep -c "host all all 0.0.0.0/0 md5")" -eq 0 ]; then
    sudo bash -c "echo 'host all all 0.0.0.0/0 md5' >> /etc/postgresql/$PG_VERSION/main/pg_hba.conf"
    sudo sed -i "/#listen_addresses/a listen_addresses = \'0.0.0.0\'" /etc/postgresql/$PG_VERSION/main/postgresql.conf
fi
sudo sed -i "s/5432/$PG_PORT/" /etc/postgresql/$PG_VERSION/main/postgresql.conf
sudo systemctl restart postgresql

echo -e "${GREEN}=> Installing rust tool chain...${END}"
command -v rustup >/dev/null 2>&1 || bash -c "curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${TOOLCHAIN}"
# https://github.com/webdevops/Dockerfile/issues/138
sed -i -re 's/^(mesg n)(.*)$/#\1\2/g' ~/.profile
source ~/.profile
rustup default nightly

echo -e "${GREEN}=> Installing diesel cli...${END}"
command -v diesel >/dev/null 2>&1 || bash -c "cargo install diesel_cli --no-default-features --features postgres --force"

echo -e "${GREEN}=> Run migrations...${END}"
diesel migration run \
    --database-url "postgres://$PG_USERNAME:$PG_PASSWORD@$PG_HOSTNAME:$PG_PORT/$PG_DB"
diesel print-schema \
    --database-url "postgres://$PG_USERNAME:$PG_PASSWORD@$PG_HOSTNAME:$PG_PORT/$PG_DB"

echo -e "${GREEN}=> Finished!${END}"
