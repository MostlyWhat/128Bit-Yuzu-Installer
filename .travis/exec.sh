#!/bin/bash -ex

# the UID for the container yuzu user is 1027
docker run -u root -v $(pwd):/liftinstall -t yuzuemu/build-environments:linux-liftinstall /bin/bash /liftinstall/.travis/build.sh
