FROM archlinux

# Update, install packages
RUN pacman -Syu --noconfirm
RUN pacman -S nginx arch-install-scripts rust bash jq --noconfirm

RUN mkdir -p /etc/nginx/logs/
ENV LC_ALL="en_US.UTF-8"

# Move app 
COPY nginx.conf /etc/nginx/nginx.conf
COPY mime.types /etc/nginx/mime.types


RUN mkdir /web_app
WORKDIR /web_app
COPY . .
RUN cargo build --release 
RUN mkdir -p data/user-home

RUN useradd gtk-user

VOLUME /var/run/docker.sock
VOLUME /web_app/data/user-home

CMD ["bash", "/web_app/kickoff.sh"]


