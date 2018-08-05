#!/bin/bash
set -e

docker stop some-postgres || true && docker rm some-postgres || true &&
docker system prune -f &&
docker volume prune -f &&
docker run --name some-postgres -d -e POSTGRES_PASSWORD= -p 5432:5432 postgres &&
sleep 5 &&
PGPASSWORD= PGOPTIONS='--client-min-messages=warning' psql -h localhost -U postgres -f dump.sql
#PGPASSWORD= PGOPTIONS='--client-min-messages=warning' psql -v ON_ERROR_STOP=1 --pset pager=off -h localhost -U postgres -f dump.data.sql

