//! Entry point for adversarial bypass tests.
//!
//! These tests treat ActionGuard's enforcement boundary as an attack surface:
//! every known bypass technique has a test here, so a future regression fails
//! CI instead of shipping.
//!
//! See `tests/bypass/README.md` for the full matrix and known blind spots.

mod bypass;
