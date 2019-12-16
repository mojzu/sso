#!/bin/bash
set -e

# TODO(fix): Signals from host not received.
# <https://hynek.me/articles/docker-signals/>

# Execute arguments.
exec $@
