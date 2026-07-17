# Command Line Interface

`automesh` is used primarily as a **command line interface (CLI)**: a single
`automesh` binary with one subcommand per operation — [`convert`](cli/convert.md),
[`defeature`](cli/defeature.md), [`diff`](cli/diff.md), [`extract`](cli/extract.md),
[`mesh`](cli/mesh.md), [`metrics`](cli/metrics.md), [`remesh`](cli/remesh.md),
[`segment`](cli/segment.md), and [`smooth`](cli/smooth.md).  Each subcommand
reads one or more input files and writes an output file.

```sh
automesh --help
<!-- cmdrun automesh --help -->
```

## Global Options

Two options apply to every subcommand: `--quiet` and `--log`.

* `--quiet` (`-q`) suppresses the terminal output — the banner, and each
  command's `Reading`/`Meshing`/`Writing`/`Done`/`Total` progress lines — while
  the command still runs normally.

* `--log <FILE>` mirrors that same terminal output into a file, with the
  color escape codes stripped so the file is plain text.  `automesh` inserts a
  local date-time stamp before the file extension, so rerunning a command with
  the same `--log` path never overwrites an earlier log:

  ```sh
  automesh --log run.log mesh hex -i octahedron.npy -o octahedron.inp -r 0
  ```

  ```sh
  Logging to run_2026-07-17T09-30-58-0600.log
  ```

  The stamp is `YYYY-MM-DD` (year-month-day), `T` marking the start of the
  time-of-day, then `HH-MM-SS` (hour-minute-second, local time), followed by
  the local UTC offset `-0600` (six hours behind UTC — Mountain Daylight
  Time, in this example).  Hyphens separate the hour, minute, and second
  instead of the more common `:` because `:` is not a legal filename
  character on Windows.

`--quiet` and `--log` are independent, so `--quiet --log run.log` runs silently
on the terminal while still writing the full log file — useful for scripted or
batch invocations where only a record of the run is wanted.
