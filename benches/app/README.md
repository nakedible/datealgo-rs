# The application benchmark

A synthetic application that uses a date library the way real software does,
for judging changes to datealgo (`scripts/app_ab.sh` and the pull request
check) and comparing it with other libraries (`scripts/app_libs.sh`). It is
split into domains, each a kind of software, each with its own events, inputs
and implementations. Each domain has a score of its own, the change in the time
it spends in calendar code, and the overall score is their weighted mean.

## Where the domains come from

The domains come from a study of the 1438 most used crates on crates.io that
depend on chrono, time or jiff, described with its results and scripts in
`scripts/corpus/README.md`. In short:

- **Conversions dominate.** Days since an epoch to year, month and day, and
  back, behind every timestamp formatted or parsed, every database value and
  every column kernel, are by far the most frequent operations. Validating a
  date with the days of its month comes with every parse.
- **The formats.** `%Y-%m-%d` is the most used format string by far, followed
  by date and time with a space or `T`, `%Y%m%d`, `%Y%m%dT%H%M%SZ`, fractions of
  a second, offsets, and month and weekday names.
- **Each kind of software has its own pattern**, which the domains follow: what
  it converts, how often, from which epochs and formats, and on which inputs.
- **Many crates write their own calendar code**, often because the libraries
  were too slow for them: 23 have their own conversions between days and
  dates, and about 30 more their own leap year or days in month functions.

## Domains

| Domain | Weight | Events |
|---|---|---|
| `logging` | 20% | Log lines in the formats of tracing-subscriber, env_logger, flexi_logger, syslog, slog-term and access logs; rolling file checks |
| `wire` | 20% | Kubernetes objects with many RFC 3339 timestamps, protobuf timestamps to and from JSON, HTTP dates formatted and parsed (with RFC 850 and asctime), AWS signatures, email and IMAP dates, JWT claims |
| `data` | 25% | Postgres, MySQL, MSSQL, Cassandra and SQLite values; column batches through date_part, date_trunc, adding months, and CSV parsing and formatting |
| `arith` | 10% | Adding months and years with clamping, ages, period bounds, business days with holiday rules, holiday tables, relative dates, month views |
| `weeks` | 5% | ISO weeks of dates and timestamps, ISO week dates as text, week ranges, weeks in a year, US and RFC 5545 week numbers, weeks of a month |
| `schedule` | 10% | Cron next runs, RFC 5545 recurrences, conversions between UTC and local time with POSIX time zone rules |
| `files` | 10% | ZIP dates, directory listings, tar times, X.509 validity, Excel serial dates, git log dates, PDF dates |

The weights are a judgement of how much calendar work each kind of software
does in practice, from how many crates do it and how often they do it. They are
the `WEIGHT` constants of the domains.

The inputs are mostly near the present, as most dates in software are, with
birth dates, dates of the past decades, future expiry dates, the edges of the
supported ranges and some invalid dates where the domain sees them. Each kind
of event comes in several implementations, written the way the different
libraries and applications write the code around the calendar functions:
digit tables, `write!`, division, fixed-position parsers, scanners, digit masks
and splitting strings, and calls to the combined functions or to the primitive
ones.

## How it measures

Each event is handled by a function that is not inlined, with the calendar
functions inlined into it, and between events the rest of the application runs,
reading from a working set larger than L2 through code and branches of its own,
so that caches and branch predictors are in the state they would be in a real
program. Each event waits for the result of the previous one. Each domain also
runs without calendar work, with `NoCalendar`, and the difference is the time
spent in calendar code.
