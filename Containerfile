FROM debian:bullseye-slim

# metadata
ARG VCS_REF
ARG BUILD_DATE
ARG IMAGE_NAME

LABEL network.virto.image.authors="we@virto.team" \
	io.parity.image.vendor="Virto Network" \
	io.parity.image.title="${IMAGE_NAME}" \
	io.parity.image.description="virto: Decentralized marketplaces" \
	io.parity.image.source="https://github.com/virto-network/virto-node/blob/${VCS_REF}/Containerfile" \
	io.parity.image.revision="${VCS_REF}" \
	io.parity.image.created="${BUILD_DATE}" \
	io.parity.image.documentation="https://github.com/virto-network/virto-node/"

# show backtraces
ENV RUST_BACKTRACE 1

# install tools and dependencies
RUN apt-get update && \
	DEBIAN_FRONTEND=noninteractive apt-get install -y \
		libssl1.1 \
		ca-certificates \
		curl && \
# apt cleanup
	apt-get autoremove -y && \
	apt-get clean && \
	find /var/lib/apt/lists/ -type f -not -name lock -delete; \
# add user and link ~/.local/share/virto to /data
	useradd -m -u 1000 -U -s /bin/sh -d /virto virto && \
	mkdir -p /data /virto/.local/share && \
	chown -R virto:virto /data && \
	ln -s /data /virto/.local/share/virto

# add virto binary to container image
COPY ./bin/virto /usr/local/bin/

USER virto

# check if executable works in this container
# RUN /usr/local/bin/virto --version

EXPOSE 30333 9933 9944
VOLUME ["/virto"]

ENTRYPOINT ["/usr/local/bin/virto"]
