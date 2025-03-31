<div align="center">
  <pre>
|| |
  </pre>
</div>

# orfile

> Or get it from a file!

Orfile is a standard for loading CLI parameters from different sources. 

## `where` and `using`
Orfile allows the developer to program `where` and `using` subcommand variants are tied into the same logic, but accept different parameters.

> [!NOTE]
> A helpful pattern is to check command requirements with `where` and then develop with `using`.

- **`where`**: Explicitly requires parameters to be passed in as args. This is best for when you're learning to use a given command, or want to see what is necessary to run a command.
- **`using`**: Allows parameters to be passed in a hierarchy from environment variables, to config files, to command line args **in order of override.** This is useful for production settings. The subcommand will still validate the config.

**Example**
The below is and example command supported from this directory by building the `tool` binary. 

```bash
tool add where --left 1 --right 2 
ADD_LEFT=1 tool add using --config ./examples/config.json -- --right 4
```

where

- `ADD_LEFT`: is an environment variable that supplies the parameter to the `--left` keyword argument.
-- `./config.json`: is a JSON formatted config that may contain any of the parameters.
-- `-- --right`: is a final override with a command line argument. 

## Contributing and getting started

| Task | Description |
|------|-------------|
| [Upcoming Events](https://github.com/movementlabsxyz/ffs/issues?q=is%3Aissue%20state%3Aopen%20label%3Apriority%3Ahigh%2Cpriority%3Amedium%20label%3Aevent) | High-priority `event` issues with planned completion dates. |
| [Release Candidates](https://github.com/movementlabsxyz/ffs/issues?q=is%3Aissue%20state%3Aopen%20label%3Arelease-candidate) | Feature-complete versions linked to events. |
| [Features & Bugs](https://github.com/movementlabsxyz/ffs/issues?q=is%3Aissue%20state%3Aopen%20label%3Afeature%2Cbug%20label%3Apriority%3Aurgent%2Cpriority%3Ahigh) | High-priority `feature` and `bug` issues. |

Please see the [CONTRIBUTING.md](CONTRIBUTING.md) file for additional contribution guidelines.

## Organization

There are five subdirectories which progressively build on one another for node logic.

1. [`orfile`](./orfile/): contains all core `orfile` logic.