#/bin/sh
docker build -f Dockerfile .. -t registry.theplanet.ca/telemetry-http-server:1.0
#docker run registry.theplanet.ca/telemetry-http-server:1.0
docker push registry.theplanet.ca/telemetry-http-server:1.0