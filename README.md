[![Crates.io](https://img.shields.io/crates/v/datealgo)](https://crates.io/crates/datealgo)
[![docs.rs](https://img.shields.io/docsrs/datealgo)](https://docs.rs/datealgo/latest/datealgo/)
![Crates.io](https://img.shields.io/crates/l/datealgo)
![Crates.io](https://img.shields.io/crates/d/datealgo)
[![GitHub Workflow Status](https://github.com/nakedible/datealgo-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/nakedible/datealgo-rs/actions/workflows/ci.yml)
![Maintenance](https://img.shields.io/maintenance/yes/2023)

# datealgo

Low-level date algorithms for libraries

This library aims to provide the **highest performance algorithms** for date
manipulation in an unopinionated way. It is meant to be used by the various
date and time libraries which can then provide ergonomic and opinionated
interfaces for their users.

## Usage

The primary contribution of this crate for date libraries are the
conversions between a day number from Unix epoch (January 1st, 1970) and a
Gregorian date:

```rust
use datealgo::{rd_to_date, date_to_rd};

assert_eq!(date_to_rd((1970, 1, 1)), 0);
assert_eq!(date_to_rd((2023, 5, 12)), 19489);
assert_eq!(rd_to_date(19489), (2023, 5, 12));
```

For convenience, there is also converters to and from Unix timestamps:

```rust
use datealgo::{secs_to_datetime, datetime_to_secs};

assert_eq!(datetime_to_secs((1970, 1, 1, 0, 0, 0)), 0);
assert_eq!(datetime_to_secs((2023, 5, 20, 9, 24, 38)), 1684574678);
assert_eq!(secs_to_datetime(1684574678), (2023, 5, 20, 9, 24, 38));
```

If the `std` feature is enabled, there are also converters to and from
`SystemTime`:

```rust
use datealgo::{systemtime_to_datetime, datetime_to_systemtime};
use std::time::{Duration, UNIX_EPOCH};

assert_eq!(systemtime_to_datetime(UNIX_EPOCH), Some((1970, 1, 1, 0, 0, 0, 0)));
assert_eq!(systemtime_to_datetime(UNIX_EPOCH + Duration::from_secs(1684574678)), Some((2023, 5, 20, 9, 24, 38, 0)));
assert_eq!(datetime_to_systemtime((2023, 5, 20, 9, 24, 38, 0)), UNIX_EPOCH.checked_add(Duration::from_secs(1684574678)));
```

## Features

The crate works in `no_std` environments and has no allocations. Most of the
functions also work in constant contexts.

- `std` (default): Include `SystemTime` conversions

## Background

There are many date and time libraries for Rust for varying use cases as the
standard library doesn't include any utilities for dealing with dates. Most
of these libraries contain their own copies of date algorithms, most
prominently the conversion from days since an epoch to a Gregorian calendar
date (year, month, day). These algorithms have been sourced from various
places with various licenses, often translated either by machine or by hand
from C algorithms found in different libc variants. The algorithms are
usually somewhat optimized for performance, but fall short of fastest
algorithms available.

## Notes

The library does not expose any kind of `Date` or `DateTime` structures, but
simply tuples for the necessary values. Bounds checking is done via
`debug_assert` only, which means the methods are guaranteed to not panic in
release builds. Callers are required to do their own bounds checking.
Datatypes are selected as the smallest that will fit the value.

Currently the library implements algorithms for the [Proleptic Gregorian
Calendar](https://en.wikipedia.org/wiki/Proleptic_Gregorian_calendar) which
is our current calendar extended backwards indefinitely. The Gregorian
calendar defines the average year to be 365.2425 days long by defining every
fourth year to be a leap year, unless the year is divisible by 100 and not
by 400.

The algorithms do not account for leap seconds, as is customary for [Unix
time](https://en.wikipedia.org/wiki/Unix_time). Every day is exactly 86400
in length, and the calculated times do not adjust for leap seconds between
timestamps.

We define [Rata Die](https://en.wikipedia.org/wiki/Rata_Die) to be integral
day numbers counted from 1st of January, 1970, which is the Unix epoch. We
use the abbreviation `rd` to concisely refer to such values. This differs
from the epoch originally chosen by Howard Jacobson, but is more convenient
for usage.

The Rata Die values are represented as `i32` for performance reasons. The
needed calculations reduce that to roughly an effective `i30` integer range,
which means a usable range of roughly -1,460,000 to 1,460,000 years.

## Benchmarks

Results on GitHub Codespaces 8-core VM:

| Function | [datealgo](https://github.com/nakedible/datealgo-rs) | [hinnant](https://howardhinnant.github.io/date_algorithms.html) | [httpdate](https://github.com/pyfisch/httpdate) | [humantime](https://github.com/tailhook/humantime) | [time](https://github.com/time-rs/time) | [chrono](https://github.com/chronotope/chrono) |
| ---------------------- | ------------- | --------- | --------- | --------- | --------- | --------- |
| date_to_rd | **2.3 ns** | 4 ns | 3.2 ns | 3.2 ns | 17.7 ns | 7.2 ns |
| rd_to_date | **3.6 ns** | 9.3 ns | 11.3 ns | 11.3 ns | 18.8 ns | 8.2 ns |
| datetime_to_systemtime | **8.6 ns** | | 9.9 ns | 9.8 ns | 57.3 ns | 50.1 ns |
| systemtime_to_datetime | **14.2 ns** | | 18.9 ns | 19 ns | 54.6 ns | 226.4 ns |

Some code has been adapted from the libraries to produce comparable
benchmarks.

## Acknowledgements

I do not claim original research on anything that is in this crate.

- [Cassio Neri and Lorenz
  Schneider](https://onlinelibrary.wiley.com/doi/full/10.1002/spe.3172):
  While searching for best method for date conversion, I stumbled upon a
  research paper which explains a novel way to optimize the performance.
  These algorithms have been implemented here based on the published
  article. This wouldn't be the best performing date conversion library
  without their work.
- [David Hinnant](https://howardhinnant.github.io/date_algorithms.html):
  While searching for "perpetual calendar" algorithms, and having already
  started my library, I stumbled upon a very similar idea by David Hinnant.
  It remains one of the cleanest and simplest algorithms while still
  retaining excellent performance.
- [Rich
  Felker](https://git.musl-libc.org/cgit/musl/tree/src/time/__secs_to_tm.c):
  The original musl `__time_to_tm` function has spread far and wide and been
  translated to many languages, and is still the basis of many of the
  standalone implementations littered among the libraries.
- [Many authors of newlib
  `gmtime_r.c`](https://sourceware.org/git/?p=newlib-cygwin.git;a=blob;f=newlib/libc/time/gmtime_r.c;hb=HEAD):
  The newlib implementation has evolved significantly over time and has now
  been updated based on the work by David Hinnant.

## Releases

See [CHANGELOG](CHANGELOG.md)

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
