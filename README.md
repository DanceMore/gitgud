# gitgud

a little utility to help make sure your `~/code` folder is tidy and pushed.

it checks all subdirectories of a given directory for Git repositories and prints a warning message if any repository is in a dirty, incomplete state, or in-progress state... so you can GITGUD and finish all your work :)

checks for:
* untracked files
* changes not staged
* branch not set to `master` / `main` (useful when you do a lot of `feature-*` work)

the `--debug` flag can be used to print debug information.



### TODO
* add args / flags to skip checks (probably clap4-ify)
* allow master/main default branch names to be configurable ..?
