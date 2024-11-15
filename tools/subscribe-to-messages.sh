#!/bin/sh

graphql-proxy subscribe-to-messages \
    -e ws://localhost:8000/admin-api/graphql-ws \
    -r 1s
