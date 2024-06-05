# gourd

**Gourd** is a command-line tool that schedules parallel runs for algorithm comparisons.

Given the parameters of the experiment, a number of test datasets,
and algorithm implementations to compare, **Gourd** runs the experiment in parallel
and provides many options for processing its results.

While originally envisioned for the DelftBlue supercomputer at
Delft University of Technology, **Gourd** can replicate the experiment on
any cluster computer with the _Slurm_ scheduler, on any UNIX-like system,
and on Microsoft Windows.

# Installation

TODO: LINUX/MAC/WINDOWS PACKAGES.

# Building

Build the software from source using `cargo build --release`.

This will produce `./target/release/gourd` and `./target/release/gourd_wrapper`.

Put these in your `$PATH`.

# Obtaining the manual

To build user and maintainer documentation run `cargo build --release --no-default-features --features documentation -vv`.

These are built into HTML, PDF, and `man` formats.

To build code documentation run `cargo doc --release --document-private-items`.

This is built into HTML.

## Installation

### For UNIX-like systems:

The user guide is a man page. Install it to a `MANPATH`, a list of which you can find by typing `echo $MANPATH`, using the following commands:

`cp target/release/manpages/gourd.1.man <manpath>/man1/gourd.1`
`cp target/release/manpages/gourd.toml.5.man <manpath>/man1/gourd.toml.5`
`cp target/release/manpages/gourd-tutorial.7.man <manpath>/man1/gourd-tutorial.7`

The user guide is then accessible using `man gourd`.

# Contributing

The maintainer documentation PDF is available as a build artifact on GitLab.
You can also compile from source (requires XeLaTeX), is placed in `target/release/manpages/maintainer.pdf`
