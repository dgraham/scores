# String similarity scoring

Mostly intended for use with Vim's [CtrlP][] fuzzy file name matcher, `scores`
ranks a set of file paths, provided on standard input, against a search term
to find the closest match.

[CtrlP]: https://github.com/ctrlpvim/ctrlp.vim

## Usage

```
$ scores -h
$ scores --limit 10 user.rb < items.cache
```

## Development

```
$ cargo test
$ cargo build --release
```

## License

Distributed under the MIT license. See LICENSE for details.
