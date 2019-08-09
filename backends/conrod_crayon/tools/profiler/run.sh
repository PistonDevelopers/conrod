USAGE="Usage: sudo ./tools/profiler/run.sh [EXECUTABLE_FILE] [OUT_FILE]"

if [ "$#" -ne 2 ]; then
    echo $USAGE
    exit 1
fi

EXECUTABLE_FILE=$1
OUT_FILE=$2
SCRIPT_FILE="$( cd "$(dirname "$0")" ; pwd -P )"
NAME=$(basename $EXECUTABLE_FILE)

set -e
rm -f out.stacks
dtrace -c $EXECUTABLE_FILE -o out.stacks -n "profile-30 /execname == \"${NAME}\"/ { @[ustack(1000)] = count(); }"
$SCRIPT_FILE/scripts/stackcollapse.pl out.stacks | $SCRIPT_FILE/scripts/flamegraph.pl > $OUT_FILE
rm -f out.stacks