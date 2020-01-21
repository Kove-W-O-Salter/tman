# TMAN [![Build Status](https://travis-ci.com/Kove-W-O-Salter/tman.svg?branch=master)](https://travis-ci.com/Kove-W-O-Salter/tman)

## BUILD
### Prerequisites.
* `rust`.
* `cargo`.
### Like a good boy.
```
cargo install tman
```
And your good to go.
### Like a masochist.
```bash
git clone https://github.com/kove-w-o-salter/tman
cd ./tman
cargo build --release
cp ./target/release/tman ~/.local/bin/tman
```

## USAGE
```
$ tman --help
USAGE:
    tman <ACTION>

ACTIONS:
    --delete             -D    <FILE_1>...    Delete specified files
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
  ```bash
  tman -D test{,1,2,3}.txt
  ```
  ```bash
  tman -R test{,1,2,3}.txt
  ```

## SETTINGS
Settings are stored in `~/.tman/settings.json`. The current available settings are:
* `use_unicode`: set to `true` if you want to see unicode characters in your output, otherwise set to `false`. **Defaults to `false`**.
* `use_colors`: set to `true` if you want to see ANSI formatting in your output, otherwise set to `false`. **Defaults to `false`**.

## CONTRIBUTING
### I would love to hear what you think.
### I am not an experienced develper.

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
