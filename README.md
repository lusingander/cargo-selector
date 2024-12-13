# cargo-selector

[![Crate Status](https://img.shields.io/crates/v/cargo-selector.svg)](https://crates.io/crates/cargo-selector)

Cargo subcommand to select and execute binary/example targets

<img src="./img/demo.gif" width=800>

(This demo uses [Ratatui](https://github.com/ratatui-org/ratatui) as an example!)

## Installation

```
$ cargo install cargo-selector
```

### AUR

```
$ paru -S cargo-selector
```

## Usage

```
Usage: cargo selector [OPTIONS]

Options:
  -i, --inline                   Display list inline
  -n, --inline-list-size <SIZE>  List size [default: 10]
  -k, --kind <NAME>              Target kind [possible values: bin, example]
  -t, --match-type <TYPE>        Match type [default: substring] [possible values: substring, fuzzy]
  -h, --help                     Print help
  -V, --version                  Print version
```

Run the command in the cargo project directory:

```
$ cargo selector
```

Then, target list will be displayed, and you can execute the following command by selecting it.

```sh
# if the target is bin
$ cargo run --bin xyz [--features "foo bar"]

# if the target is example
$ cargo run --example xyz [--features "foo bar"]
```

By switching the action, you can also run only the build.

### Keybindings

| Key                               | Description                                  |
| --------------------------------- | -------------------------------------------- |
| <kbd>Down</kbd> <kbd>Ctrl+n</kbd> | cursor down                                  |
| <kbd>Up</kbd> <kbd>Ctrl+p</kbd>   | cursor up                                    |
| <kbd>Enter</kbd>                  | execute `cargo run --bin/example <selected>` |
| <kbd>Tab</kbd>                    | switch actions                               |
| <kbd>Esc</kbd> <kbd>Ctrl+c</kbd>  | quit                                         |

## License

MIT
