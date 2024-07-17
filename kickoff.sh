#!/bin/sh
cd /web_app
echo "Starting Nginx on port 80"
nginx 


# Use provided values or fallback to defaults
IMAGE_NAME="${IMAGE_NAME:-"bws-mgr/vroot"}"
IMAGE_TAG="${IMAGE_TAG:-"latest"}"
COMMON_NETWORK="${COMMON_NETWORK:-"bws-mgr-net"}"
SHARED_VOLUME_NAME="${SHARED_VOLUME_NAME:-"/web_app/data/user-home"}"

DOCKER_API_URL="http:/v1.41/images/json"

# Build image, if not exists
IMAGE_EXISTS=$(curl --silent --unix-socket /var/run/docker.sock ${DOCKER_API_URL} | jq -r --arg IMAGE_NAME "${IMAGE_NAME}" --arg IMAGE_TAG "${IMAGE_TAG}" '[.[] | select(.RepoTags != null) | .RepoTags[]] | any(. == "\($IMAGE_NAME):\($IMAGE_TAG)")')
if [ "${IMAGE_EXISTS}" != "true" ]; then
    if [ "$IMAGE_NAME:$IMAGE_TAG" != "bws-mgr/vroot:latest" ]; then
        echo "Vroot image not found, and the requested image is not the default one. Aborting..."
        exit 1
    fi
    echo "Vroot image not found, building..."
    cd vroot_config
    TAR_FILE="context.tar"
    tar --exclude=$TAR_FILE -cf $TAR_FILE .
    curl --unix-socket /var/run/docker.sock -X POST \
    -H "Content-Type: application/tar" \
    -H "Content-Type: application/x-tar" \
    -H "X-Registry-Auth: {}" \
    --data-binary @$TAR_FILE \
    -v "http://localhost/build?t=$IMAGE_NAME:$IMAGE_TAG&dockerfile=dockerfile"
    rm $TAR_FILE
    cd ..
fi

# Create network, if not exists
NETWORKS_URL="http:/v1.41/networks"
NETWORK_EXISTS=$(curl --silent --unix-socket /var/run/docker.sock ${NETWORKS_URL} | jq -r --arg COMMON_NETWORK "${COMMON_NETWORK}" '.[] | select(.Name == $COMMON_NETWORK) | .Name')

if [ -z "${NETWORK_EXISTS}" ]; then
    echo "Network not found, asumming we are running whit --network=host"    
fi

if [ "$MODE" == "dev" ]; then
    echo "Installing dev dependencies"
    pacman -R rust --noconfirm
    pacman -Syu --noconfirm
    pacman -S openssh git rustup vim which npm diesel-cli docker bind --noconfirm
    rustup default stable
    echo "Setting up ssh"
    passwd -d root
    ssh-keygen -A
    cat >/etc/ssh/sshd_config <<EOL
PermitRootLogin yes
PasswordAuthentication yes
PermitEmptyPasswords yes
Subsystem       sftp    /usr/lib/ssh/sftp-server
EOL

    /sbin/sshd -f /etc/ssh/sshd_config
    echo "Running in dev mode, not starting web_api"
    while [ true ]; do
        sleep 1
    done
fi;
COMMON_NETWORK=${COMMON_NETWORK} SHARED_VOLUME_NAME=${SHARED_VOLUME_NAME} VROOT_IMAGE="${IMAGE_NAME}:${IMAGE_TAG}" /web_app/target/release/web_api

