# Profiler

This script is a simple wrapper of [dtrace](http://dtrace.org/) and [flamegraph](https://github.com/brendangregg/FlameGraph).

```sh
$ # Notes that dtrace requires additional privileges.
$ sudo ./tools/profiler/run.sh target/release/saturn out.svg
```