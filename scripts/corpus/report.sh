#!/usr/bin/env bash
# Report how the crates downloaded by scripts/corpus/fetch.py use date
# libraries: which conversions, fields, arithmetic, formats and parsers they
# call, which format strings they use, and which crates write their own
# calendar code. Needs ripgrep.
#
# usage: scripts/corpus/report.sh [DIR]   DIR defaults to target/corpus
#
# Searches library sources only, not tests, benches or examples. The counts
# are of matching lines and of crates with a matching line. Names such as
# `.year()` may also match other APIs, so the counts are approximate.
set -euo pipefail
cd "${1:-target/corpus}"

# Lines that may be about dates, extracted once, as searching all sources for
# each pattern takes minutes
if [ ! -f lines.txt ] || [ src -nt lines.txt ]; then
  (cd src && rg --no-heading -n --type rust -g '!tests/**' -g '!benches/**' -g '!examples/**' -i \
    -e 'chrono|jiff|time::|datetime|date|timestamp|utc|unix_epoch|systemtime|weekday|month|year|iso_week' \
    -e 'rfc3339|rfc2822|strftime|ymd|hms|ordinal|duration|julian|epoch|tz|offset|leap|calendar|cron|schedule' \
    >../lines.txt || true)
fi

# Print the number of matching lines and crates for each pattern
count() {
  local pat
  for pat in "$@"; do
    rg -N -e "$pat" lines.txt | awk -F/ '{n++; c[$1] = 1} END {k = 0; for (x in c) k++; printf "%7d %6d  ", n, k}'
    echo "$pat"
  done
}

echo "$(jq length crates.json 2>/dev/null || python3 -c 'import json; print(len(json.load(open("crates.json"))))') crates, $(wc -l <lines.txt) lines that may be about dates"
echo
echo "  lines crates"
echo "Timestamps to dates and times"
count 'from_timestamp(_opt|_millis|_micros|_nanos|_secs)?\(' 'timestamp_opt\(' 'from_unix_timestamp(_nanos)?\(' \
  'Timestamp::from_(second|millisecond|microsecond|nanosecond)' 'Utc::now\(\)' 'Local::now\(\)' \
  'OffsetDateTime::now_utc\(\)' 'SystemTime::now\(\)' 'UNIX_EPOCH'
echo "Dates and times to timestamps"
count '\.timestamp\(\)' '\.timestamp_(millis|micros|nanos)(_opt)?\(\)' '\.unix_timestamp(_nanos)?\(\)' \
  'duration_since\(\s*(std::time::)?UNIX_EPOCH' '\.and_hms(_opt|_milli_opt|_micro_opt|_nano_opt)?\('
echo "Constructing dates"
count 'from_ymd(_opt)?\(' 'from_calendar_date\(' 'from_yo(_opt)?\(|from_ordinal_date\(' \
  'from_isoywd(_opt)?\(|from_iso_week_date\(' 'from_num_days_from_ce(_opt)?\(' 'from_julian_day\(' 'civil::date\('
echo "Fields"
count '\.year\(\)' '\.month\(\)' '\.day\(\)' '\.weekday\(\)' '\.ordinal0?\(\)' '\.iso_week\(\)|to_iso_week_date\(\)' \
  'num_days_from_(monday|sunday)\(\)|number_from_(monday|sunday)\(\)' \
  'days_in_month|num_days_in_month|days_in_year|month_length' 'is_leap_year|leap_year'
echo "Arithmetic"
count 'checked_(add|sub)_months|Months::new' 'checked_(add|sub)_days|Days::new' \
  'succ_opt|pred_opt|next_day\(\)|previous_day\(\)' '(Duration|TimeDelta)::(days|weeks)\(' \
  'signed_duration_since|\.since\(|\.until\(' 'trunc_subsecs|duration_trunc|duration_round' \
  'start_of_(day|week|month)|beginning_of|first_day|last_day|end_of_month' 'nth_weekday|weekday_of_month'
echo "Formatting"
count '\.format\("' 'to_rfc3339(_opts)?\(' 'to_rfc2822\(' 'format_description!|format_description::parse' \
  'well_known::Rfc3339|Rfc3339' 'fmt_http_date|httpdate|HttpDate'
echo "Parsing"
count 'parse_from_rfc3339' 'parse_from_rfc2822' 'parse_from_str\(' \
  '(OffsetDateTime|PrimitiveDateTime|Date|UtcDateTime)::parse\(' 'parse::<(DateTime|NaiveDate|NaiveDateTime|Timestamp|Zoned)'
echo "Time zones"
count 'chrono_tz|Tz::' 'with_timezone\(' 'FixedOffset' 'UtcOffset' 'from_local_datetime|from_utc_datetime|offset_from_utc'
echo "Calendar code of their own"
count '146097|146_097' '719468|719_468|719163|719_163' 'days_from_civil|civil_from_days' '2440588|2_440_588' \
  'fn is_leap' 'fn days_in_month|fn month_days|fn days_of_month' '\[(0|31), (31|28|59|0)'
echo
echo "Crates with their own conversions between days and dates:"
rg -N -l -e '146097|146_097|719468|719_468|days_from_civil|civil_from_days' --type rust src 2>/dev/null |
  awk -F/ '{print $2}' | sort -u | tr '\n' ' ' | fold -s -w 100
echo
echo
echo "The most used strftime-style format strings:"
rg -N -o -e '(format|parse_from_str|strftime|strptime|parse)\(\s*(&?[a-z_]+,\s*)?"([^"]*%[^"]*)"' -r '$3' lines.txt |
  sort | uniq -c | sort -rn | head -30
echo
echo "The most used format descriptions of the time crate:"
rg -N -o -e 'format_description!\(\s*"([^"]*)"' -r '$1' lines.txt | sort | uniq -c | sort -rn | head -15
