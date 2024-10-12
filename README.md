# Rusty GOTO (Rgoto)

`rgoto` is a Rust-based command-line tool that reads your SSH configuration file (`~/.ssh/config`), presents a list of configured hosts, and allows you to select one to SSH into interactively.
This work is partially inspired by [goto](https://github.com/grafviktor/goto).
Also, this project borrows and modifies some codes from [cliclack](https://github.com/fadeevab/cliclack) to support `hjkl` and index based navigation.

## Features

- Automatically detects and parses your `~/.ssh/config` file.
- Displays a list of configured SSH hosts for easy selection.
- Supports reading usernames specified in the SSH config file.
- Uses the system's `ssh` command to establish an interactive SSH session.
- (New) now supports latency checks! Just select the first option and hit Enter.

## Installation

- One-Liner

    ```sh
    cargo install --git https://github.com/JiwanChung/rgoto --locked
    ```

1. Ensure you have Rust installed. You can install Rust using [rustup](https://rustup.rs/).

2. Build the project:

    ```sh
    cargo build --release
    ```

3. Move the executable to appropriate places pointed by `$PATH`.

    ```sh
    cp ./target/release/rgoto $HOME/.local/bin
    ```
  

1. Run the project:

    ```sh
    rgoto
    ```


2. Follow the prompts to select a host from your SSH config and connect.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue for any bugs or feature requests.
