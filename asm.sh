#!/bin/sh

set -e

gb='https://godbolt.org/api/compiler/nightly/compile?options=-C%20opt-level=3&addFilters=trim'

egrep -v '^[/#]' src/lib.rs | curl "$gb" --data-binary @- > lib.asm
