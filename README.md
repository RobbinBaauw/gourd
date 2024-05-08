# gourd

**Gourd** is a command-line tool that schedules parallel runs for algorithm comparisons.
Given the parameters of the experiment, a number of test datasets,
and algorithm implementations to compare,
**Gourd**
runs the experiment in parallel and provides many options for
processing its results. While originally envisioned for the DelftBlue
supercomputer at Delft University of Technology,
**Gourd**
can replicate the experiment on any cluster computer with the
_Slurm_
scheduler, on any UNIX-like system, and on Microsoft Windows.

# Installation

Build the software from source using `cargo build --release` and run the
produced binary at `./target/release/gourd`.

A full list of requirements will be added.

## Obtaining the manual

### For UNIX-like systems:

The user guide is a man page. Install it to a `MANPATH`, a list of which you can find by typing `echo $MANPATH`, using the following command:

`cp docs/user/gourd.man <manpath>/man1/gourd.1`

The user guide is then accessible using `man gourd`.

# Contributing

A maintainer documentation PDF is available as a build artifact on GitLab.
You can also compile from source (requires LaTeX):

`cd docs/maintainer && ./compile.sh`
