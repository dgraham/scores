# String similarity scoring

Mostly intended for use with Vim's [CtrlP][] fuzzy file name matcher, `scores`
ranks a set of file paths, provided on standard input, against a search term
to find the closest match.

Install the [vim-scores][] plugin to use this as CtrlP's file name matcher.

[CtrlP]: https://github.com/ctrlpvim/ctrlp.vim
[vim-scores]: https://github.com/dgraham/vim-scores

## Usage

```
$ scores --limit 10 user.rb < items.cache
```

```
$ scores -h
String similarity scoring

Usage:
    scores term [options]

Options:
    -h, --help           Print this help message
    -V, --version        Print version information
    -l, --limit MAX      Maximum number of results
    -E, --exclude NAME   Exclude the specified entry
    -r, --reference NAME A reference to match against
```

## Development

```
$ cargo test
$ cargo build --release
```

## License

Distributed under the MIT license. See LICENSE for details.
