// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::f64;
use core::num::flt2dec::strategy::grisu::Fp;
use core::num::dec2flt::rawfp::{fp_to_float, prev_float, next_float, round_normal};

#[test]
fn fp_to_float_half_to_even() {
    fn is_normalized(sig: u64) -> bool {
            // intentionally written without {min,max}_sig() as a sanity check
            sig >> 52 == 1 && sig >> 53 == 0
    }

    fn conv(sig: u64) -> u64 {
        // The significands are perfectly in range, so the exponent should not matter
        let (m1, e1, _) = fp_to_float::<f64>(Fp { f: sig, e: 0 }).integer_decode();
        assert_eq!(e1, 0 + 64 - 53);
        let (m2, e2, _) = fp_to_float::<f64>(Fp { f: sig, e: 55 }).integer_decode();
        assert_eq!(e2, 55 + 64 - 53);
        assert_eq!(m2, m1);
        let (m3, e3, _) = fp_to_float::<f64>(Fp { f: sig, e: -78 }).integer_decode();
        assert_eq!(e3, -78 + 64 - 53);
        assert_eq!(m3, m2);
        m3
    }

    let odd = 0x1F_EDCB_A012_345F;
    let even = odd - 1;
    assert!(is_normalized(odd));
    assert!(is_normalized(even));
    assert_eq!(conv(odd << 11), odd);
    assert_eq!(conv(even << 11), even);
    assert_eq!(conv(odd << 11 | 1 << 10), odd + 1);
    assert_eq!(conv(even << 11 | 1 << 10), even);
    assert_eq!(conv(even << 11 | 1 << 10 | 1), even + 1);
    assert_eq!(conv(odd << 11 | 1 << 9), odd);
    assert_eq!(conv(even << 11 | 1 << 9), even);
    assert_eq!(conv(odd << 11 | 0x7FF), odd + 1);
    assert_eq!(conv(even << 11 | 0x7FF), even + 1);
    assert_eq!(conv(odd << 11 | 0x3FF), odd);
    assert_eq!(conv(even << 11 | 0x3FF), even);
}

#[test]
fn integers_to_f64() {
    assert_eq!(fp_to_float::<f64>(Fp { f: 1, e: 0 }), 1.0);
    assert_eq!(fp_to_float::<f64>(Fp { f: 42, e: 7 }), (42 << 7) as f64);
    assert_eq!(fp_to_float::<f64>(Fp { f: 1 << 20, e: 30 }), (1u64 << 50) as f64);
    assert_eq!(fp_to_float::<f64>(Fp { f: 4, e: -3 }), 0.5);
}

const SOME_FLOATS: [f64; 9] =
    [0.1f64, 33.568, 42.1e-5, 777.0e9, 1.1111, 0.347997,
     9843579834.35892, 12456.0e-150, 54389573.0e-150];


#[test]
fn human_f64_roundtrip() {
    for &x in &SOME_FLOATS {
        let (f, e, _) = x.integer_decode();
        let fp = Fp { f: f, e: e};
        assert_eq!(fp_to_float::<f64>(fp), x);
    }
}

#[test]
fn rounding_overflow() {
    let x = Fp { f: 0xFF_FF_FF_FF_FF_FF_FF_00u64, e: 42 };
    let rounded = round_normal::<f64>(x);
    let adjusted_k = x.e + 64 - 53;
    assert_eq!(rounded.sig, 1 << 52);
    assert_eq!(rounded.k, adjusted_k + 1);
}

#[test]
fn prev_float_monotonic() {
    let mut x = 1.0;
    for _ in 0..100 {
        let x1 = prev_float(x);
        assert!(x1 < x);
        assert!(x - x1 < 1e-15);
        x = x1;
    }
}

const MIN_SUBNORMAL: f64 = 5e-324;

#[test]
fn next_float_zero() {
    let tiny = next_float(0.0);
    assert_eq!(tiny, MIN_SUBNORMAL);
    assert!(tiny != 0.0);
}

#[test]
fn next_float_subnormal() {
    let second = next_float(MIN_SUBNORMAL);
    // For subnormals, MIN_SUBNORMAL is the ULP
    assert!(second != MIN_SUBNORMAL);
    assert!(second > 0.0);
    assert_eq!(second - MIN_SUBNORMAL, MIN_SUBNORMAL);
}

#[test]
fn next_float_inf() {
    assert_eq!(next_float(f64::MAX), f64::INFINITY);
    assert_eq!(next_float(f64::INFINITY), f64::INFINITY);
}

#[test]
fn next_prev_identity() {
    for &x in &SOME_FLOATS {
        assert_eq!(prev_float(next_float(x)), x);
        assert_eq!(prev_float(prev_float(next_float(next_float(x)))), x);
        assert_eq!(next_float(prev_float(x)), x);
        assert_eq!(next_float(next_float(prev_float(prev_float(x)))), x);
    }
}

#[test]
fn next_float_monotonic() {
    let mut x = 0.49999999999999;
    assert!(x < 0.5);
    for _ in 0..200 {
        let x1 = next_float(x);
        assert!(x1 > x);
        assert!(x1 - x < 1e-15, "next_float_monotonic: delta = {:?}", x1 - x);
        x = x1;
    }
    assert!(x > 0.5);
}
