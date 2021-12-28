# Dups - fast search for duplicated directories

dups is CLI tool that searches different directories which contains the same files.

I build this tool to declatter my photo/video collection which had more than 50k files and ocuppied 300GB of disk space.

For comparing files it uses CRC32 algorithm and checks file sizes. By default it reads only first 1024 bytes of each file and show directories containing at least 10 files in common. This behaviour could be configured with command line arguments (see options).


## Install
1. Install Rust https://www.rust-lang.org/tools/install
2. Build from source
```
git clone git@github.com:un1t/dups.git
cd dups
cargo build --release
```
target/release/**dups** is a binary executable file that now can be copied to any place.

## Usage
1. Basic usage:
```
$ dups ~/Pictures
```

2. Multiple locations:
```
$ dups ~/Pictures ~/Documents
```

3. Minimum file size:
```
$ dups ~/Pictures -m 100KB
```

## Help

```
USAGE:
    dups <directories>... --head <N> --min-intersection <N> --min-size <N>

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -h, --head <N>                Reads only N bytes to calculate checksum. Set 0 to read full file. [default: 1024]
    -i, --min-intersection <N>    How many equal files must be in 2 directories to consider those directories as
                                  duplicates [default: 10]
    -m, --min-size <N>            Ignore files which is smaller than this size [default: 1]

ARGS:
    <directories>...    Directories to search
```