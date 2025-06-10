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

> [!TIP]
>  To see how to implement such a CLI tool using the `orfile::Orfile` macro, see [`tool::cli::add`](./examples/tool/src/cli/add/mod.rs).

**Example**
The below is and example command supported from this directory by building the `tool` binary. 

```bash
tool add where --left 1 --right 2 
ADD_LEFT=1 tool add using --args-path ./examples/config.json -- --right 4
```

## `slect`
The `orfile` repo also houses the `slect` API which used for chosing one of many subcommand as Selections. 

> [!WARNING]
> The task of prefixing flattend args in [`clap`](https://github.com/clap-rs/clap) has long been unresolved, owing to abstractions on the parser and their availability in different contexts.
> 
> `slect` works around these limitations at the expense of direct `--help` support--essentially extending the parsing into `extra_args` with prefix handling.  
>
> **See, for more information about complications associated with `clap` prefixing:**
> - https://github.com/clap-rs/clap/issues/3117
> - https://github.com/clap-rs/clap/issues/5374

> [!TIP]
> To see how to implement such a CLI tool using the `slect::Slect` macro, see [`select_tool::cli`](./examples/select-tool/src/cli/mod.rs).

**Example**

Select everything: 
```
select-tool --add --multiply --divide -- --add.left 2 --add.right 2 --multiply.left 2 --multiply.right 3 --divide.left 3 --divide.right 2
Add { left: 2, right: 2 }
4
Multiply { left: 2, right: 3 }
6
Divide { left: 3, right: 2 }
1
```

Select one:
```
select-tool --add -- --add.left 2 --add.right 2
Add { left: 2, right: 2 }
4
```

Select multiple but fail to fulfill one:
```
select-tool --add --multiply --divide -- --add.left 2 --add.right 2 --multiply.left 2 --multiply.right 3 --divide.left 3
Error: Failed to parse subcommand: error: the following required arguments were not provided:
  --right <RIGHT>

Usage: divide --left <LEFT> --right <RIGHT>

For more information, try '--help'.
```

Get some help with selections:
```
select-tool --help-all
Usage: select-tool [EXTRA_ARGS]...

Arguments:
  [EXTRA_ARGS]...  

Options:
  -h, --help  Print help

Selection (1/3): add
The arguments for the add command

Usage: add{} --left <LEFT> --right <RIGHT>

Options:
      --left <LEFT>    The left number
      --right <RIGHT>  The right number
  -h, --help           Print help (see more with '--help')

Selection (2/3): multiply
The arguments for the multiply command

Usage: multiply{} --left <LEFT> --right <RIGHT>

Options:
      --left <LEFT>    The left number
      --right <RIGHT>  The right number
  -h, --help           Print help (see more with '--help')

Selection (3/3): divide
The arguments for the divide command

Usage: divide{} --left <LEFT> --right <RIGHT>

Options:
      --left <LEFT>    The left number
      --right <RIGHT>  The right number
  -h, --help           Print help (see more with '--help')
```

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