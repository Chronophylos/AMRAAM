#!/bin/bash

CONTAINER="amraamtest"

cargo build
docker container run -itd --rm --name $CONTAINER amraam:v1 bash >/dev/null
docker container cp target/debug/amraam "$CONTAINER:amraam" >/dev/null
docker container exec -it "$CONTAINER" amraam "$*"
docker container stop "$CONTAINER" >/dev/null
docker container rm "$CONTAINER" >/dev/null
