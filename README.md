# TRASH
Safely delete and restore files.

## BUILD
If you've got `rust` installed with `rustup` then just:
```bash
git clone https://github.com/kove-w-o-salter/trash-rs
cd ./trash-rs
cargo build --release
cp ./target/release/trash ~/.local/bin/trash
```
and you're good to go.

## USAGE
* Move several files or directories to the trash.
  ```
  trash delete FILE_1...
  ```
* Restore several files from the trash to their original locations:
  ```
  trash restore FILE_1...
  ```
* List the items in the trash that, optionally, match the regex PATTERN:
  ```
  trash list [PATTERN]
  ```
* Permenantly delete all items in the trash including their original locations:
  ```
  trash empty
  ```

## EXAMPLES
* Move `test.txt`, `test1.txt`, `test2.txt` and `test3.txt` to the trash run:
  ```bash
  trash delete test{,1,2,3}.txt
  ```
* Restore `test.txt`, `test1.txt`, `test2.txt` and `test3.txt` to the trash run:
  ```bash
  trash restore test{,1,2,3}.txt
  ```

## TODO
- [X] File/directory restoration.
- [X] Regex searching
- [X] Propper error messages.
- [ ] Custom trash directory.
- [ ] Debug output.
- [ ] Error logging.