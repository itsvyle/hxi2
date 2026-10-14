import "utils.just"
default:
    just --list
    
init:
    @just _utils-check-program "dprint" 0
    @just _utils-check-program "gofmt" 0
    @just _utils-check-program "rustfmt" 0

format:
    dprint fmt

clean:
    @just _utils-check-program "fd" 2
    fd --type=directory -H -I ".venv" -x rm -rf {}
    fd --type=directory -H -I node_modules -x rm -rf {}
    fd --type=directory -H -I dist -x rm -rf {}
    fd --type=directory -H -I target -x bash -c '[[ -f "{//}/Cargo.toml" ]] && echo "Removing {}" && rm -rf {}'

compile-proto is_docker="false":
    @just _utils-check-program "buf" 2
    @just _utils-check-program "cargo" 2
    buf build -o ./generated-proto/hxi2.binpb
    buf generate --debug
    {{if is_docker == "false" {"cd generated-proto/parse-perms && cargo run --locked"} else {""} }}

docker-compile-proto:
    @just _utils-check-program "cargo" 2
    docker build -f ./Dockerfile.compile-proto -t hxi2-proto-builder .
    docker run --rm \
        --user $(id -u):$(id -g) \
        -v "$(pwd)/generated-proto:/app/generated-proto" \
        hxi2-proto-builder
    cd generated-proto/parse-perms && cargo run --locked

grpcui url="0.0.0.0:8080":
    grpcui -plaintext -protoset ./generated-proto/hxi2.binpb {{url}}