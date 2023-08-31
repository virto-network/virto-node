set shell := ["nu", "-c"]
podman := `(which podman) ++ (which docker) | (first).path` # use podman otherwise docker
ver := `open node/Cargo.toml | get package.version`
image := "ghcr.io/virto-network/virto"
node := "target/release/virto-node"
chain := "kreivo"
rol := "collator"
relay := "kusama"

alias b := build-local
alias c := check
alias t := test

_task-selector:
	#!/usr/bin/env nu
	let selected_task = (
		just --summary -u | split row ' ' | to text | fzf --header 'Available Virto recipes' --header-first --layout reverse --preview 'just --show {}' |
		if ($in | is-empty) { 'about' } else { $in }
	)
	just $selected_task

@about:
	open node/Cargo.toml | get package | table -c

@version:
	echo {{ ver }}

@list-crates:
	open Cargo.toml | get workspace.members | each { open ($in + /Cargo.toml) | get package.name } | str join "\n"

@_check_deps:
	rustup component add clippy

check: _check_deps
	cargo clippy --all-targets -- --deny warnings
	cargo fmt --all -- --check


@test crate="" *rest="":
	cargo test (if not ("{{crate}}" | is-empty) { "-p" } else {""}) {{crate}} {{ rest }}

build-local features="":
	cargo build --release --features '{{features}}'

build-container:
	#!/usr/bin/env nu
	'FROM docker.io/paritytech/ci-linux:production as builder
	WORKDIR /virto
	COPY . /virto
	RUN cargo build --release

	FROM debian:bookworm-slim
	VOLUME /data
	COPY --from=builder /virto/{{ node }} /usr/bin
	ENTRYPOINT ["/usr/bin/virto-node"]
	CMD ["--dev"]'
	| {{ podman }} build . -t {{ image }}:{{ ver }} --ignorefile .build-container-ignore -f -

# Used to speed things up when the build environment is the same as the container(debian)
build-container-local: build-local
	#!/usr/bin/env nu
	'FROM debian:bookworm-slim
	LABEL io.containers.autoupdate="registry"
	VOLUME /data
	COPY {{ node }} /usr/bin
	ENTRYPOINT ["/usr/bin/virto-node"]
	CMD ["--dev"]'
	| {{ podman }} build . -t {{ image }}:{{ ver }} -t {{ image }}:latest -f -

container_args := "--base-path /data '$NODE_ARGS'" + if rol == "collator" {
	" --collator --force-authoring -- --chain " + relay
} else { "" }
container_name := chain + "-" + rol

create-container:
	@^mkdir -p release
	podman rm -f {{ container_name }}
	podman create --name {{ container_name }} --volume {{ container_name }}-data:/data {{ image }} {{ container_args }}
	podman generate systemd --name --new --no-header --env 'NODE_ARGS=' {{ container_name }} | str replace -s '$$' '$' | save -f release/container-{{ chain }}-{{ rol }}.service

_chain_artifacts:
	@^mkdir -p release
	{{ node }} export-genesis-state --chain {{ chain }} | save -f release/{{ chain }}_genesis
	{{ node }} export-genesis-wasm --chain {{ chain }} | save -f release/{{ chain }}_genesis.wasm
	{{ node }} build-spec --disable-default-bootnode --chain {{ chain }} | save -f release/{{ chain }}_chainspec.json

release-artifacts: build-local create-container _chain_artifacts

release-tag:
	git tag {{ ver }}

_zufix := os() + if os() == "linux" { "-x64" } else { "" }
zombienet network="": build-local
	#!/usr/bin/env nu
	# Run zombienet with a profile from the `zombienet/` folder chosen interactively
	mut net = "{{ network }}"
	if "{{ network }}" == "" {
		let net_list = (ls zombienet | get name | path basename | str replace .toml '')
		$net = ($net_list | to text | fzf --preview 'open {}.toml' | if ($in | is-empty) { $net_list | first } else { $in })
	}
	bin/zombienet-{{ _zufix }} -p native spawn $"zombienet/($net).toml"

get-zombienet-dependencies: (_get-latest "zombienet" "zombienet-"+_zufix) (_get-latest "polkadot" "polkadot") (_get-latest "cumulus" "polkadot-parachain")

_get-latest repo bin:
	#!/usr/bin/env nu
	^mkdir -p bin
	http get https://api.github.com/repos/paritytech/{{ repo }}/releases
	# cumulus has two kinds of releases, we exclude runtimes
	| where "tag_name" !~ "parachains" | first | get assets_url | http get $in
	| where name =~ {{ bin }} | first | get browser_download_url
	| http get $in --raw | save bin/{{ bin }} --progress --force
	chmod u+x bin/{{ bin }}
