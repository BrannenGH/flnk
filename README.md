# flnk

An enhancement to the Unix `ln` command, providing recursive directory linking capabilities.

## Description

`flnk` extends the functionality of the traditional `ln` command by enabling recursive hard linking and symlinking of entire directory structures. It's particularly useful when you need to create hard links or symbolic links for entire directory trees.

## Usage

```bash
flnk [OPTION]... [-T] TARGET LINK_NAME
flnk [OPTION]... TARGET
flnk [OPTION]... TARGET... DIRECTORY
flnk [OPTION]... -t DIRECTORY TARGET...
```

### Options

- `-s, --symbolic`: Create symbolic links instead of hard links
- `-f, --force`: Remove existing destination files
- `-b`: Make a backup of each existing destination file
- `-r, --relative`: Create relative symbolic links
- `-v, --verbose`: Print name of each linked file
- `-u`: Run in interactive TUI mode

## License

This project is licensed under the MIT License - see the LICENSE file for details.