//! Nesting-depth guards and parse-thread options (tex2math 2.x).
//!
//! MathML **render** is heap-iterative and does not use these guards.

use crate::ast::UnknownCommandPolicy;
use std::cell::Cell;

/// Default maximum parse nesting depth.
pub const DEFAULT_MAX_NESTING_DEPTH: u32 = 64;

/// Re-export name used by older call sites / docs.
pub const MAX_NESTING_DEPTH: u32 = DEFAULT_MAX_NESTING_DEPTH;

thread_local! {
    static PARSE_DEPTH: Cell<u32> = const { Cell::new(0) };
    static PARSE_MAX: Cell<u32> = const { Cell::new(DEFAULT_MAX_NESTING_DEPTH) };
    static PARSE_DEPTH_EXCEEDED: Cell<bool> = const { Cell::new(false) };
    static UNKNOWN_COMMAND: Cell<UnknownCommandPolicy> =
        const { Cell::new(UnknownCommandPolicy::Identifier) };
}

/// Install parse options for the current thread (call at the start of `parse`).
pub fn configure_parse(max_depth: u32, unknown_command: UnknownCommandPolicy) {
    PARSE_MAX.with(|c| c.set(max_depth.max(1)));
    PARSE_DEPTH.with(|c| c.set(0));
    PARSE_DEPTH_EXCEEDED.with(|c| c.set(false));
    UNKNOWN_COMMAND.with(|c| c.set(unknown_command));
}

/// Current unknown-command policy (set by [`configure_parse`]).
#[inline]
pub fn unknown_command_policy() -> UnknownCommandPolicy {
    UNKNOWN_COMMAND.with(Cell::get)
}

/// No-op retained for call sites that previously configured render depth.
/// MathML emission is heap-iterative and does not use a native stack budget.
pub fn configure_render(_max_depth: u32) {}

/// RAII parse-depth guard.
pub struct DepthGuard;

impl DepthGuard {
    /// Enter a parse frame. Returns `Err(())` if the limit would be exceeded.
    #[inline]
    pub fn enter_parse() -> Result<Self, ()> {
        PARSE_DEPTH.with(|d| {
            let max = PARSE_MAX.with(Cell::get);
            let cur = d.get();
            if cur >= max {
                PARSE_DEPTH_EXCEEDED.with(|f| f.set(true));
                return Err(());
            }
            d.set(cur + 1);
            Ok(DepthGuard)
        })
    }
}

impl Drop for DepthGuard {
    fn drop(&mut self) {
        PARSE_DEPTH.with(|c| c.set(c.get().saturating_sub(1)));
    }
}

#[inline]
pub fn take_parse_depth_exceeded() -> bool {
    PARSE_DEPTH_EXCEEDED.with(|f| f.replace(false))
}

#[inline]
pub fn parse_depth_exceeded() -> bool {
    PARSE_DEPTH_EXCEEDED.with(|f| f.get())
}

#[inline]
pub fn mark_parse_depth_exceeded() {
    PARSE_DEPTH_EXCEEDED.with(|f| f.set(true));
}

#[inline]
pub fn nesting_depth_error_message() -> String {
    let max = PARSE_MAX.with(Cell::get);
    format!("Maximum nesting depth ({max}) exceeded")
}
