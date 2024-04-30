# Contribution guidelines

# Setup your repository

Before doing anything issue the command:
`git config --local core.hooksPath .hooks/`

Otherwise your commits will get automatically rejected by the CI.

# Code style

The rules are simple.

If `cargo fmt` produces something different you are wrong.

This is pursuant to certain exceptions but they have to be well motivated
in the MR description.

# Contribution style

Limit the amount of commits if possible. All commits should be lowercase.
