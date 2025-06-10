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