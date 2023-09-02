#!/bin/sh

set -e

echo '//! | Function | [datealgo](https://github.com/nakedible/datealgo-rs) | [hinnant](https://howardhinnant.github.io/date_algorithms.html) | [httpdate](https://github.com/pyfisch/httpdate) | [humantime](https://github.com/tailhook/humantime) | [time](https://github.com/time-rs/time) | [chrono](https://github.com/chronotope/chrono) |'
echo '//! | ---------------------- | ------------- | --------- | --------- | --------- | --------- | --------- |'
for group in date_to_rd rd_to_date datetime_to_systemtime systemtime_to_datetime; do
    echo -n "//! | $group |"
    for fun in datealgo hinnant httpdate humantime time chrono; do
        fn=target/criterion/compare_$group/$fun/new/estimates.json
        if [ -e "$fn" ]; then
            v=$(jq '.mean.point_estimate * 10 | round / 10'  < "$fn")
            if [ "$fun" = "datealgo" ]; then
                echo -n " **$v ns** |"
            else
                echo -n " $v ns |"
            fi
        else
            echo -n " |"
        fi
    done
    echo
done
echo
