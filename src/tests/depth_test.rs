use crate::{parse_latex, parse_math, MAX_NESTING_DEPTH};
use winnow::Parser;

/// Pathological nesting must fail with a structured error, not SIGABRT / stack overflow.
#[test]
fn test_ultra_deep_fraction_nesting_is_rejected() {
    let depth = 5000usize;
    let mut input = String::with_capacity(depth * 8 + 8);
    for _ in 0..depth {
        input.push_str("\\frac{1}{");
    }
    input.push('2');
    for _ in 0..depth {
        input.push('}');
    }

    let mut cursor = input.as_str();
    let res = parse_math.parse_next(&mut cursor);
    assert!(
        res.is_err(),
        "Deep nesting should hit the recursion limit and return Err"
    );

    // High-level facade should surface a stable message.
    let err = parse_latex(&input).expect_err("parse_latex should reject ultra-deep nesting");
    assert!(
        err.message.contains("nesting depth")
            || err.message.contains(&MAX_NESTING_DEPTH.to_string()),
        "unexpected error message: {}",
        err.message
    );
}

/// Nesting just under the limit should still parse (smoke check that the cap is not tiny).
#[test]
fn test_moderate_nesting_still_parses() {
    let depth = 32usize;
    let mut input = String::new();
    for _ in 0..depth {
        input.push_str("\\frac{1}{");
    }
    input.push('x');
    for _ in 0..depth {
        input.push('}');
    }
    parse_latex(&input).expect("moderate nesting should parse");
}
