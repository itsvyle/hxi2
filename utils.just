set unstable
export VOLTA_FEATURE_PNPM := '1'

# Checks if a program is installed, prints a warning if not
# Valid values for `required`:
# 0: Prints warning if not found, but does not exit
# 1: Prints error if not found, but does not exit
# 2: Prints error if not found and exits with code 1
_utils-check-program name required="1":
    @command -v {{name}} >/dev/null 2>&1 || { echo >&2 "{{ if required == "1" { style("error") + "ERROR" } else if required == "2" { style("error") + "FATAL ERROR" } else { style("warning") + "WARNING" } }}: \`{{name}}\` is {{ if required == "1" { "required" } else if required == "2" { "required" } else { "recommended" } }} to contribute to the project, but couldn't be found in PATH{{NORMAL}}" {{ if required == "2" { "&& exit 1" } else { "" } }};}