//! Nesting-depth guards and parse-thread options (tex2math 2.x).
//!
//! MathML **render** is heap-iterative and does not use these guards.
//!
//! # Parse context
//!
//! Combinators still read max-depth / unknown-command policy from thread-local
//! cells so signatures stay `&mut &str` (zero cursor overhead). Install via
//! [`ParseCtx::install`], which returns a [`ParseCtxGuard`] that **restores**
//! the previous TLS snapshot on drop — nested `parse` calls on the same thread
//! are therefore safe (outer options/depth resume after the inner call).
//!
//! Parallel parses on separate threads remain independent.

use crate::ast::UnknownCommandPolicy;
use std::cell::Cell;

/// Default maximum parse nesting depth.
pub const DEFAULT_MAX_NESTING_DEPTH: u32 = 64;

/// Re-export name used by older call sites / docs.
pub const MAX_NESTING_DEPTH: u32 = DEFAULT_MAX_NESTING_DEPTH;

/// Explicit parse context (preferred entry for [`crate::parse`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseCtx {
    pub max_depth: u32,
    pub unknown_command: UnknownCommandPolicy,
    pub recovery: crate::ast::RecoveryMode,
}

impl Default for ParseCtx {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_NESTING_DEPTH,
            unknown_command: UnknownCommandPolicy::Identifier,
            recovery: crate::ast::RecoveryMode::Tolerant,
        }
    }
}

impl ParseCtx {
    /// Install this context for the current thread and reset depth counters.
    ///
    /// Returns a guard that restores the previous TLS values when dropped.
    #[must_use]
    pub fn install(self) -> ParseCtxGuard {
        ParseCtxGuard::push(self)
    }

    /// Build from public [`crate::ParseOptions`].
    #[inline]
    pub fn from_parse_options(opts: &crate::ParseOptions) -> Self {
        Self {
            max_depth: opts.max_depth,
            unknown_command: opts.unknown_command,
            recovery: opts.recovery,
        }
    }
}

/// RAII guard for a nested [`ParseCtx`] install (restores prior TLS on drop).
#[derive(Debug)]
pub struct ParseCtxGuard {
    prev_depth: u32,
    prev_max: u32,
    prev_exceeded: bool,
    prev_unknown: UnknownCommandPolicy,
    prev_recovery: crate::ast::RecoveryMode,
}

impl ParseCtxGuard {
    fn push(ctx: ParseCtx) -> Self {
        let prev_depth = PARSE_DEPTH.with(Cell::get);
        let prev_max = PARSE_MAX.with(Cell::get);
        let prev_exceeded = PARSE_DEPTH_EXCEEDED.with(Cell::get);
        let prev_unknown = UNKNOWN_COMMAND.with(Cell::get);
        let prev_recovery = RECOVERY.with(Cell::get);

        PARSE_MAX.with(|c| c.set(ctx.max_depth.max(1)));
        PARSE_DEPTH.with(|c| c.set(0));
        PARSE_DEPTH_EXCEEDED.with(|c| c.set(false));
        UNKNOWN_COMMAND.with(|c| c.set(ctx.unknown_command));
        RECOVERY.with(|c| c.set(ctx.recovery));

        Self {
            prev_depth,
            prev_max,
            prev_exceeded,
            prev_unknown,
            prev_recovery,
        }
    }
}

impl Drop for ParseCtxGuard {
    fn drop(&mut self) {
        PARSE_DEPTH.with(|c| c.set(self.prev_depth));
        PARSE_MAX.with(|c| c.set(self.prev_max));
        PARSE_DEPTH_EXCEEDED.with(|c| c.set(self.prev_exceeded));
        UNKNOWN_COMMAND.with(|c| c.set(self.prev_unknown));
        RECOVERY.with(|c| c.set(self.prev_recovery));
    }
}

thread_local! {
    static PARSE_DEPTH: Cell<u32> = const { Cell::new(0) };
    static PARSE_MAX: Cell<u32> = const { Cell::new(DEFAULT_MAX_NESTING_DEPTH) };
    static PARSE_DEPTH_EXCEEDED: Cell<bool> = const { Cell::new(false) };
    static UNKNOWN_COMMAND: Cell<UnknownCommandPolicy> =
        const { Cell::new(UnknownCommandPolicy::Identifier) };
    static RECOVERY: Cell<crate::ast::RecoveryMode> =
        const { Cell::new(crate::ast::RecoveryMode::Tolerant) };
}

/// Current recovery mode (set by [`ParseCtx::install`]).
#[inline]
pub fn recovery_mode() -> crate::ast::RecoveryMode {
    RECOVERY.with(Cell::get)
}

/// Current unknown-command policy (set by [`ParseCtx::install`]).
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
