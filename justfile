set shell := ["nu", "-c"]
docker := `(which podman) ++ (which docker) | (first).path`
ver := `open node/Cargo.toml | get package.version`

@list-tasks:
	just --list

@about:
	open node/Cargo.toml | get package | table -c

@_check_deps:
	rustup component add clippy

check: _check_deps
	cargo clippy --features runtime-benchmarks --all-targets --workspace -- --deny warnings
	cargo fmt --all -- --check

test:
	cargo test

build-local:
	cargo build --release

build-container tag=ver:
	{{docker}} build . -t virto-network/virto:{{ tag }}

_zufix := os() + if os() == "linux" { "-x64" } else { "" }
zombienet network="":
	#!/usr/bin/env nu
	mut net = "{{ network }}"
	# interactive selection with fuzzy find
	if "{{ network }}" == "" { 
		$net = (ls zombienet | get name | path basename | str replace .toml '' | to text | fzf)
	}
	bin/zombienet-{{ _zufix }} -p native spawn $"zombienet/($net).toml"

get-zombienet-dependencies: (_get-latest "zombienet" "zombienet-"+_zufix) (_get-latest "polkadot" "polkadot") (_get-latest "cumulus" "polkadot-parachain")

_get-latest repo bin:
	#!/usr/bin/env nu
	^mkdir -p bin
	(
		http get https://api.github.com/repos/paritytech/{{ repo }}/releases
		# cumulus has two kinds of releases, we exclude runtimes
		| where "tag_name" !~ "parachains" | first | get assets_url | http get $in
		| where name =~ {{ bin }} | first | get browser_download_url
		| http get $in --raw | save bin/{{ bin }} --progress --force
	)
	chmod u+x bin/{{ bin }}
