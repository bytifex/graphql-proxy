#!/bin/sh

SCRIPT_DIR=$(dirname $0)

graphql-proxy query -e http://localhost:8000/admin-api/graphql -q $SCRIPT_DIR/queries/get-config.graphql
