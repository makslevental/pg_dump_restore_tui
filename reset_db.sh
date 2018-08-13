#!/bin/bash
set -e

docker stop some-postgres || true && docker rm some-postgres || true &&
docker system prune -f &&
docker volume prune -f &&
docker build -t plpythonu .
docker run --name some-postgres -d -e POSTGRES_PASSWORD=mysecretpassword -p 5432:5432 plpythonu &&
sleep 5 &&
PGPASSWORD=mysecretpassword PGOPTIONS='--client-min-messages=warning' psql -v ON_ERROR_STOP=1 --pset pager=off -h localhost -U postgres -f dump.schema.sql
PGPASSWORD=mysecretpassword PGOPTIONS='--client-min-messages=warning' psql -v ON_ERROR_STOP=1 --pset pager=off -h localhost -U postgres -f trigger_rewrite.sql
PGPASSWORD=mysecretpassword PGOPTIONS='--client-min-messages=warning' psql -v ON_ERROR_STOP=1 --pset pager=off -h localhost -U postgres -f dump.data.sql

# docker exec -i -t loving_heisenberg /bin/bash #by Name
