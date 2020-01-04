# TRASH-RS
Safely delete and restore files.

![trash-rs](https://github.com/Kove-W-O-Salter/trash-rs/blob/master/preview.png?raw=true)

## BUILD
If you've got `rust` installed with `rustup` then just:
```bash
git clone https://github.com/kove-w-o-salter/trash-rs
cd ./trash-rs
cargo build --release
cp ./target/release/trash ~/.local/bin/trash
```
and you're good to go.
## BINARY
If you're on an 64 bit Ubuntu based distro you should be able to download a precompiled binary [here](https://drive.google.com/open?id=1LNJnqY3O6GdpY2ROCUg_t3BP4SK8X06I). However, this binary will not always be up to date.

## USAGE
* Move several files or directories to the trash.
  ```
  trash -D f0,f1,...,fn
  ```
* Restore several files from the trash to their original locations:
  ```
  trash -R f0,f1,...,fn
  ```
* List the items in the trash that, optionally, match the regex PATTERN:
  ```
  trash -L p
  ```
* Permenantly delete all items in the trash including their original locations:
  ```
  trash -E
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

## TODO
- [X] File/directory restoration.
- [X] Regex searching
- [X] Propper error messages.
- [ ] Custom trash directory.
- [ ] Debug output.
- [ ] Error logging.
