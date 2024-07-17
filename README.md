# Broadway Session Manager

### Introduction
This app is still in early development, thus a database with `someone` `Was` as admin user is provided by default. Additionally this app is made for linux hosts only.

This web service allows multiple concurrent users to connect to GTK4 and GTK3 Broadway sessions. For each session, a new Docker container gets started containing the user's home directory and the Broadway service with all registered GTK3/4 apps started. After a session gets closed, the used container will be destroyed (preserving the user's home). All files inside the user's home are also accessible via a basic file manager in the web UI. The files as well as sessions are protected so that only the user has access to his sessions.

Notes:
- Although it is recommended to run this web service inside a Docker container, it _should_ be possible to run it directly on the host OS. Docker, however, needs to be installed in order to create the user session containers.
- This application was NOT tested for any security vulnerabilities. Despite Docker providing isolation and measures to prevent unauthorized access, the API needs access to the Docker socket of the host machine. I do not (yet) recommend running this container in a security-sensitive environment!

# Running the Service:
### Building/Preparation
Clone this repository:
`git clone <this_repository>`

Optional: configure the session
Under `vroot_config/`, the Dockerfile can be modified to install new GTK apps. For these apps to be started, they need to be registered in the `apps.json` file.

Change into the cloned folder and build the container:
`docker build -t katzenkralle/bws-mgr:latest .`

Create a Docker network used by the service to communicate with the sessions:
`docker network create <my_bsw_network>`

### Running
For running the app with Docker, these are the recommended default settings. After starting the container the first time, it will, if not present, build the image for the sessions:
`docker run -e SHARED_VOLUME_NAME=<path_to_user_homes> -e COMMON_NETWORK=<my_bsw_network> -v <path_to_user_homes>:/web_app/data/user-home -v /var/run/docker.sock:/var/run/docker.sock --network=<my_bsw_network> -p 80:80 --name=bws-mgr katzenkralle/bws-mgr:dev`

The following environmental variables can be set:
- `IMAGE_NAME`/`IMAGE_TAG`: If not the provided and automatically built image for sessions, but a custom one should be used.
- `MODE`: `dev` or any. If `dev`, the container will not start the API automatically but will install useful apps for development in the container as well as set up an SSH server.