#!/bin/bash -e
# TIMEOUT for the startup

PROJECT_NAME=${PROJECT_NAME:-rustbier}
now=$( date '+%s' )
UNTIL=$(( $now + 70 ))

DOCKER=(docker)
DOCKER_COMPOSE=(docker-compose -p $PROJECT_NAME -f "docker-compose.yaml")

if ! docker info >/dev/null 2>&1; then
    DOCKER=(sudo -EH "${DOCKER[@]}")
    DOCKER_COMPOSE=(sudo -EH "${DOCKER_COMPOSE[@]}")
fi

"${DOCKER_COMPOSE[@]}" up -d

eval 'container_s3="$( "${DOCKER_COMPOSE[@]}" ps s3 | awk '\''BEGIN{out=""}END{print(out)}/^[^ -]/{out=out$1}'\'' )"'


# get_port "s3" "9000"
function get_port()
{
    local SERVICE="$1"
    local SERVICE_PORT="$2"
    local PORT
    PORT="$("${DOCKER_COMPOSE[@]}" port $SERVICE $SERVICE_PORT | cut -d: -f2)"
    if [ $? -ne 0 ]; then
      error
    fi

    echo "$PORT"
}

function error()
{
  >&2 echo
  >&2 echo "+++++++++++++++++++++++ LOG OUTPUT ++++++++++++++++++++++"
  "${DOCKER_COMPOSE[@]}" logs

  exit 1
}

function timeout_error()
{
  echo

  >&2 echo "ERROR: timeout reached.. $1"
  error
}

echo -n "Waiting for Minio (Fake S3) to start..."
minio_port=$(get_port s3 9000)
until curl localhost:$minio_port >/dev/null 2>&1
do
  echo -n '.'

  if [ $UNTIL -lt $(date '+%s') ]; then
    timeout_error "service Minio didn't start in time"
  fi

  sleep 0.1
done
echo

# Needs to be before Rustbier starts - configure access control for Minio
echo "Setting up Minio..."
MINIO_BUCKET=local/apollo
"${DOCKER[@]}" exec "${container_s3}" /opt/mc config host add local http://127.0.0.1:9000 apollousr apollopwd
"${DOCKER[@]}" exec "${container_s3}" /opt/mc mb --region "eu-west-1" "${MINIO_BUCKET}"
"${DOCKER[@]}" exec "${container_s3}" /opt/mc policy download "${MINIO_BUCKET}"

for f in tests/resources/*; do
    filename=$(echo $f | cut -d'/' -f3)
    cat $f | "${DOCKER[@]}" exec -i "${container_s3}" /opt/mc pipe "${MINIO_BUCKET}/$filename"
done

echo -n "Checking Minio..."
until ( curl -I -m 1 "http://localhost:${minio_port}/apollo/watermark" | grep -q 'HTTP/[0-9.]* 200 ' ) >/dev/null 2>&1; do
  echo -n '.'

  if [ $UNTIL -lt $(date '+%s') ]; then
    timeout_error "service minio didn't have the right images when we needed them"
  fi

  sleep 0.2
done
echo

