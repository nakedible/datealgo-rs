//! Formatting and parsing of dates as text, in the different styles real code
//! writes them in: digit pairs from a table into a fixed buffer, `core::fmt`
//! into a buffer, digits by division, fixed-position parsers, scanners of
//! variable fields, digit masks, and splitting strings into numbers.

use std::fmt;

pub const MONTHS: [&[u8; 3]; 12] = [
    b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov", b"Dec",
];
pub const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
/// Weekday abbreviations, Monday first
pub const WEEKDAYS: [&[u8; 3]; 7] = [b"Mon", b"Tue", b"Wed", b"Thu", b"Fri", b"Sat", b"Sun"];
pub const WEEKDAY_NAMES: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

const DIGITS: &[u8; 200] = b"\
    0001020304050607080910111213141516171819\
    2021222324252627282930313233343536373839\
    4041424344454647484950515253545556575859\
    6061626364656667686970717273747576777879\
    8081828384858687888990919293949596979899";

/// A fixed buffer to write text into
pub struct Buf<const N: usize> {
    pub b: [u8; N],
    pub len: usize,
}

impl<const N: usize> Buf<N> {
    pub fn new() -> Self {
        Buf { b: [0; N], len: 0 }
    }

    pub fn push(&mut self, c: u8) {
        if self.len < N {
            self.b[self.len] = c;
            self.len += 1;
        }
    }

    pub fn extend(&mut self, s: &[u8]) {
        for &c in s {
            self.push(c);
        }
    }

    /// Two digits from the table
    pub fn put2(&mut self, v: u32) {
        let i = v as usize % 100 * 2;
        self.push(DIGITS[i]);
        self.push(DIGITS[i + 1]);
    }

    /// Four digits from the table
    pub fn put4(&mut self, v: u32) {
        self.put2(v / 100);
        self.put2(v % 100);
    }

    /// `width` digits by division, most significant first
    pub fn put_div(&mut self, mut v: u64, width: usize) {
        let mut tmp = [b'0'; 20];
        for i in (0..width.min(20)).rev() {
            tmp[i] = b'0' + (v % 10) as u8;
            v /= 10;
        }
        self.extend(&tmp[..width.min(20)]);
    }

    /// A signed year: four digits from 0 to 9999, otherwise a sign and at
    /// least four digits, as RFC 3339 extensions and ISO 8601 do
    pub fn put_year(&mut self, y: i32) {
        if (0..=9999).contains(&y) {
            self.put4(y as u32);
        } else {
            self.push(if y < 0 { b'-' } else { b'+' });
            let a = y.unsigned_abs() as u64;
            self.put_div(a, if a > 99999 { 6 } else { 5 });
        }
    }

    /// A fraction of a second from nanoseconds with the given digits
    pub fn put_frac(&mut self, nanos: u32, digits: u32) {
        self.push(b'.');
        self.put_div((nanos / 10u32.pow(9 - digits)) as u64, digits as usize);
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.b[..self.len]
    }
}

impl<const N: usize> fmt::Write for Buf<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.extend(s.as_bytes());
        Ok(())
    }
}

/// A hash of some bytes, as the result of formatting
pub fn hash(s: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64 ^ s.len() as u64;
    let mut chunks = s.chunks_exact(8);
    for c in &mut chunks {
        h = (h ^ u64::from_le_bytes(c.try_into().unwrap()))
            .wrapping_mul(0x100_0000_01b3)
            .rotate_left(29);
    }
    for &c in chunks.remainder() {
        h = (h ^ c as u64).wrapping_mul(0x100_0000_01b3);
    }
    h
}

/// Mix a value into a hash
pub fn mix(h: u64, x: u64) -> u64 {
    (h ^ x).wrapping_mul(0x9e37_79b9_7f4a_7c15).rotate_left(23)
}

/// The value of `n` digits at `s[i..]`, if they are all digits
pub fn digits(s: &[u8], i: usize, n: usize) -> Option<u32> {
    let mut v = 0u32;
    for &c in s.get(i..i + n)? {
        let d = c.wrapping_sub(b'0');
        if d > 9 {
            return None;
        }
        v = v * 10 + d as u32;
    }
    Some(v)
}

/// A scanner over text, reading fields of variable length
pub struct Scan<'a> {
    pub s: &'a [u8],
    pub i: usize,
}

impl<'a> Scan<'a> {
    pub fn new(s: &'a [u8]) -> Self {
        Scan { s, i: 0 }
    }

    /// An unsigned number of `min..=max` digits
    pub fn num(&mut self, min: usize, max: usize) -> Option<u32> {
        let start = self.i;
        let mut v = 0u32;
        while self.i < self.s.len() && self.i - start < max {
            let d = self.s[self.i].wrapping_sub(b'0');
            if d > 9 {
                break;
            }
            v = v * 10 + d as u32;
            self.i += 1;
        }
        (self.i - start >= min).then_some(v)
    }

    /// A fraction of a second of any number of digits, as nanoseconds
    pub fn frac(&mut self) -> u32 {
        let mut nanos = 0u32;
        let mut scale = 100_000_000u32;
        while let Some(&c) = self.s.get(self.i) {
            let d = c.wrapping_sub(b'0');
            if d > 9 {
                break;
            }
            nanos += d as u32 * scale;
            scale /= 10;
            self.i += 1;
        }
        nanos
    }

    /// Whether the next byte is `c`, consuming it if it is
    pub fn eat(&mut self, c: u8) -> bool {
        if self.s.get(self.i) == Some(&c) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    /// Whether the next byte is any of `cs`, consuming it if it is
    pub fn eat_any(&mut self, cs: &[u8]) -> bool {
        match self.s.get(self.i) {
            Some(c) if cs.contains(c) => {
                self.i += 1;
                true
            }
            _ => false,
        }
    }

    pub fn skip_spaces(&mut self) {
        while self.s.get(self.i) == Some(&b' ') {
            self.i += 1;
        }
    }

    /// A month abbreviation, case-insensitively, as a number from 1 to 12
    pub fn month_name(&mut self) -> Option<u8> {
        let m = month_from_abbr(self.s.get(self.i..self.i + 3)?)?;
        self.i += 3;
        Some(m)
    }

    /// A weekday abbreviation, as a number from 1 (Monday) to 7, skipping a
    /// full name after it
    pub fn weekday_name(&mut self) -> Option<u8> {
        let w = self.s.get(self.i..self.i + 3)?;
        let w = WEEKDAYS.iter().position(|d| d.eq_ignore_ascii_case(w))? as u8 + 1;
        self.i += 3;
        while self.s.get(self.i).is_some_and(|c| c.is_ascii_alphabetic()) {
            self.i += 1;
        }
        Some(w)
    }

    /// An offset from UTC: `Z`, `z`, `±hh:mm`, `±hhmm` or `±hh`, in seconds
    pub fn offset(&mut self) -> Option<i32> {
        if self.eat_any(b"Zz") {
            return Some(0);
        }
        let sign = if self.eat(b'+') {
            1
        } else if self.eat(b'-') {
            -1
        } else {
            return None;
        };
        let h = self.num(2, 2)?;
        self.eat(b':');
        let m = self.num(0, 2).unwrap_or(0);
        (h < 24 && m < 60).then_some(sign * (h * 3600 + m * 60) as i32)
    }

    pub fn done(&self) -> bool {
        self.i == self.s.len()
    }
}

/// A month abbreviation as a number from 1 to 12, matched by its bytes
pub fn month_from_abbr(s: &[u8]) -> Option<u8> {
    let key = u32::from_le_bytes([s[0] | 0x20, s[1] | 0x20, s[2] | 0x20, 0]);
    let m = match &key.to_le_bytes()[..3] {
        b"jan" => 1,
        b"feb" => 2,
        b"mar" => 3,
        b"apr" => 4,
        b"may" => 5,
        b"jun" => 6,
        b"jul" => 7,
        b"aug" => 8,
        b"sep" => 9,
        b"oct" => 10,
        b"nov" => 11,
        b"dec" => 12,
        _ => return None,
    };
    Some(m)
}

/// Whether the bytes at the given positions are digits, by subtracting `'0'`
/// from all of them and checking the positions in one mask, as arrow-cast does
pub fn digit_mask(s: &[u8], expected: u32) -> bool {
    let mut mask = 0u32;
    for (i, &c) in s.iter().take(32).enumerate() {
        mask |= ((c.wrapping_sub(b'0') < 10) as u32) << i;
    }
    mask & expected == expected
}

/// Numbers of a string split at any of the separators, as application code
/// often parses dates
pub fn split_numbers<const N: usize>(s: &str, seps: &[char]) -> Option<[u32; N]> {
    let mut out = [0u32; N];
    let mut parts = s.split(|c| seps.contains(&c));
    for v in out.iter_mut() {
        *v = parts.next()?.parse().ok()?;
    }
    Some(out)
}
