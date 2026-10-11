# How software uses date libraries

A study of the crates on crates.io that depend on a date library, made to
design the application benchmark (`benches/app/`) on what software actually
does with dates.

## Method

`fetch.py` downloads the sources of the 1000 most downloaded dependents of
chrono, the 500 of time and the 300 of jiff, leaving out crates that only
dev-depend on them. `report.sh` searches the library sources of those crates,
not their tests, benches or examples, for the date APIs they call, the format
strings they use and the calendar code they write themselves. The most used
crates of each kind were then read to see which calendar operations run in
their hot paths, how often and on which inputs.

```text
scripts/corpus/fetch.py [DIR]     # about 2.6 GB of sources, to target/corpus by default
scripts/corpus/report.sh [DIR]    # needs ripgrep
```

The sources are only searched and read, never built or run. The numbers below
are from October 2026: 1438 crates, 870 depending on chrono, 426 on time and
281 on jiff. They count matching lines and crates with a matching line. Names
like `.year()` can match other APIs too, so the counts are approximate, but
they show what is common and what is rare.

## What the crates call

| Operation | Lines | Crates |
|---|---:|---:|
| `UNIX_EPOCH` | 3422 | 422 |
| `Utc::now()` | 3184 | 379 |
| `SystemTime::now()` | 2002 | 349 |
| `.timestamp()` | 1552 | 322 |
| `.year()` | 1446 | 266 |
| `from_timestamp*()` | 931 | 225 |
| `.month()` | 1181 | 224 |
| `duration_since(UNIX_EPOCH)` | 785 | 221 |
| `from_ymd*()` | 3043 | 206 |
| `to_rfc3339*()` | 1289 | 205 |
| `.timestamp_{millis,micros,nanos}()` | 980 | 179 |
| `parse_from_rfc3339()` | 894 | 167 |
| `parse_from_str()` | 643 | 147 |
| `.day()` | 498 | 146 |
| `.and_hms*()` | 1215 | 143 |
| `.weekday()` | 299 | 91 |
| leap year | 400 | 49 |
| days in month or year | 444 | 43 |
| start or end of day, week or month | 390 | 36 |
| `checked_{add,sub}_months()` | 95 | 25 |
| `.iso_week()` | 55 | 23 |
| `succ_opt()`, `pred_opt()`, `next_day()` | 49 | 16 |

Conversions between timestamps and dates, and constructing dates from year,
month and day, are by far the most common. Fields, RFC 3339 and parsing with
format strings follow. The days of the week, the days in a month and leap years
are used by about a tenth as many crates, and month arithmetic, ISO weeks and
stepping a day at a time by fewer still. They are concentrated in the crates
that do much calendar work: query engines, schedulers and calendars.

The most used format strings, of strftime-style ones:

| Format | Uses |
|---|---:|
| `%Y-%m-%d` | 131 |
| `%Y-%m-%d %H:%M:%S` | 59 |
| `%Y%m%d` | 33 |
| `%Y-%m-%dT%H:%M:%SZ` | 30 |
| `%H:%M:%S` | 24 |
| `%Y%m%dT%H%M%SZ` | 21 |
| `%Y-%m-%d %H:%M:%S%.6f` | 20 |
| `%Y-%m-%dT%H:%M:%S` | 16 |
| `%F` | 14 |
| `%Y-%m-%d %H:%M:%S%.f` | 12 |

The time crate's format descriptions show the same order, from
`[year]-[month]-[day]` (29 uses) down. Month and weekday names come with HTTP
dates, email, syslog and git. `%Y%m%dT%H%M%SZ` is the AWS signature and
iCalendar format.

## Calendar code of their own

Many crates write calendar code themselves, often because the libraries were
too slow or did not have what they needed:

| Code | Lines | Crates |
|---|---:|---:|
| `fn is_leap*` | 51 | 31 |
| `fn days_in_month` and alike | 61 | 29 |
| Tables of month lengths or cumulative days | 112 | 40 |
| Epoch constants 719468 or 719163 | 19 | 18 |
| `days_from_civil`, `civil_from_days` | 41 | 10 |

23 crates have their own conversions between days and dates: arrow-array,
arrow-cast, asupersync, azul-layout, datafusion-functions, dbase, deno, gstuff,
kdbplus, kreuzberg, merman-core, mise, nautilus-core, parse_datetime,
pdf_oxide, questdb-rs, satkit, serde_amqp, simple_pg_client, spider,
turso_core, umya-spreadsheet and zen-expression. Others copy musl's
`__secs_to_tm`: httpdate, der, prost-types and tracing-subscriber.

## How each kind of software uses dates

Reading the hot paths of the most used crates of each kind showed patterns the
counts do not.

**Logging and tracing** (tracing-subscriber, env_logger, flexi_logger, log4rs,
simplelog, fern, slog-term, tracing-appender, syslog). Every line formats the
present, as RFC 3339 with milliseconds or microseconds, in local time with an
offset, or as syslog's `Oct 11 14:34:56`, and none of them cache it. Rolling
file writers check on every write whether a new hour, day or week has started,
using the day of the week for weekly files, and name files by date.

**Web, cloud and authentication** (hyper, cookie, actix-web, aws-sigv4,
aws-smithy-types, k8s-openapi, azure and Google SDKs, openidconnect). HTTP
dates are formatted for Last-Modified and cookie expiry and parsed from
conditional requests and cookies, with the day of the week, month names and
two-digit years. Servers cache the Date header per second. AWS signatures
format the date twice per request. Kubernetes objects carry 5 to 20 RFC 3339
timestamps each, parsed on every list and watch event, the heaviest parsing
load. Token validation turns each claim into a date and time.

**Serialization** (prost-types, pbjson-types, serde_with, bson, toml). Protobuf
timestamps go to and from RFC 3339 with offsets, fractions and leap seconds
clamped, over the full range of years; BSON converts milliseconds; serde_with
converts seconds and milliseconds through dates.

**Database drivers** (sqlx, postgres-types, diesel, mysql_common, tiberius,
scylla, clickhouse, duckdb, rusqlite). Every value is converted between a date
and the driver's epoch: days and microseconds since 2000-01-01 for Postgres,
days since 0001-01-01 and 1900-01-01 for MSSQL, days with 1970-01-01 at 2^31
for Cassandra, separate fields for MySQL, text in several formats tried in turn
for SQLite. Many rebuild the epoch date on every call.

**Columnar engines** (arrow, parquet, datafusion, polars, iceberg). Kernels run
over whole columns: extracting the year, month, day, day of the week, day of
the year, quarter and ISO week, truncating to a day, week, month, quarter or
year, adding months with the day clamped to the end of the month, and parsing
and formatting dates. DataFusion and Polars wrote their own calendar code for
these, as chrono was too slow.

**Certificates** (der, x509-parser, asn1-rs, rcgen, rustls-acme). UTCTime and
GeneralizedTime are parsed, validated and converted to seconds twice per
certificate, with years 1950-2049 for UTCTime; der converts each time three
times.

**Files and documents** (zip, cab, eza, lsd, gix-date, calamine,
rust_xlsxwriter, umya-spreadsheet, email, imap, lopdf, pdf_oxide, typst). ZIP
entries carry MS-DOS dates validated per entry; directory listings show the
time or the year by age; git shows dates with the day of the week and month
names; spreadsheets convert serial dates with Excel's 1900 leap year bug; email
and IMAP dates are parsed with names and offsets; PDF dates have optional
fields.

**Scheduling, calendars and time zones** (cron, croner, saffron,
tokio-cron-scheduler, rrule, chrono-tz, zoneinfo64, time-tz, bdays, chronoutil,
icu_calendar). Cron engines search for the next run day by day, checking the
day of the month and of the week, with the days of each month. RFC 5545
recurrences find the nth or last day of the week of months and number weeks
from a configurable first day. Time zone code evaluates rules such as "the
second Sunday of March" for every conversion past the last transition.
Business day calendars step a day at a time and evaluate holiday rules,
including Easter. Relative dates add months with clamping and find the start
and end of weeks, months, quarters and years.

Across all of these, most dates are near the present: log lines and requests
now, values within years of now, birth dates, and expiry dates up to years
ahead. The edges of the supported range appear as sentinels such as
9999-12-31.

## In the benchmark

`benches/app/` has a domain for each of these kinds of software, with the
operations, formats and inputs above: logging; web, cloud, serialization and
messaging formats (`wire`); database drivers and columnar engines (`data`);
calendar arithmetic and business rules (`arith`); weeks; scheduling and time
zones (`schedule`); and files, certificates and documents (`files`). Their
weights in the overall score are a judgement of how much calendar work each
kind of software does, from how many crates do it and how often.
