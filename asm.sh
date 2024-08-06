#!/bin/sh

set -e

funs="
rd_to_date
date_to_rd
rd_to_weekday
date_to_weekday
next_date
prev_date
secs_to_dhms
dhms_to_secs
secs_to_datetime
datetime_to_secs
is_leap_year
days_in_month
rd_to_isoweekdate
isoweekdate_to_rd
date_to_isoweekdate
isoweekdate_to_date
isoweeks_in_year
systemtime_to_secs
secs_to_systemtime
systemtime_to_datetime
datetime_to_systemtime
"

if [ "$1" = "--update" ]; then
    update=1
fi

for fn in $funs
do
    if [ -n "$update" ]; then
        cargo asm --features=asmdump --no-color --lib datealgo::asm::$fn | grep '\S' > asm/$fn.asm
    else
        cargo asm --features=asmdump --no-color --lib datealgo::asm::$fn | grep '\S' | diff -wu asm/$fn.asm -
    fi
done
