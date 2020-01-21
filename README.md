# TRASH [![Build Status](https://travis-ci.com/Kove-W-O-Salter/trash.svg?branch=master)](https://travis-ci.com/Kove-W-O-Salter/trash)
Safely delete and restore files and directories.

## BUILD
If you've got `rust` installed with `rustup` then just:
```bash
git clone https://github.com/kove-w-o-salter/trash
cd ./trash
cargo build --release
cp ./target/release/trash ~/.local/bin/trash
```
and you're good to go.

## USAGE
```
$ trash --help
USAGE:
    trash <ACTION>

ACTIONS:
    --delete             -D    <FILE_1>...    Trash specified files
    --restore            -R    <FILE>         Restore specified file
        --origin         -o    <PATH>         Set the origin
        --version        -v                   Set the revision
            <VERSION>                         Use a specific version
            latest                            Use the newest version (default)
            all                               Use all versions
    --list               -L                   List items in the trash
        --pattern        -p    <REGEX>        Set the search pattern
        --simple         -p                   Set the simple mode
    --empty              -E                   Permenantly delete trash content
```

## EXAMPLES
* Move `test.txt`, `test1.txt`, `test2.txt` and `test3.txt` to the trash run:
  ```bash
  trash -D test{,1,2,3}.txt
  ```
* Restore `test.txt`, `test1.txt`, `test2.txt` and `test3.txt` to the trash run:
  ```bash
  trash -R test{,1,2,3}.txt
  ```

## SETTINGS
Settings are stored in `~/.trash/settings.json`. The current available settings are:
* `use_unicode`: set to `true` if you want to see unicode characters in your output, otherwise set to `false`. **Defaults to `false`**.
* `use_colors`: set to `true` if you want to see ANSI formatting in your output, otherwise set to `false`. **Defaults to `false`**.

## CONTRIBUTING
### I would love to hear what you think.
Any feedback or contributions will be examined and potentially accepted A.S.A.P. So feel free to open **issues** or **PR**s.
### I am not an experienced develper.
I'm simply a high-school student with an enjoyment of coding and computer science. As such, I cannot guarentee the code quality of this repo.

## TODO
- [X] File/directory restoration.
- [X] Regex searching
- [X] Propper error messages.
- [X] Configuration file.
- [ ] Colored error messages.
- [ ] Progress bars.
- [ ] Custom trash directory.
- [ ] Debug output.
- [ ] Error logging.
