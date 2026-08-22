// SPDX-License-Identifier: Apache-2.0

//! The terminal-only redraw bookkeeping (slice S069, FR-002).
//!
//! Tracks how many lines the previous frame occupied, so the next frame can
//! erase exactly that many before writing over them, per
//! `contracts/status-block.md`'s redraw sequence: `\x1b[<n>A` (cursor up `n`
//! lines) then `\x1b[0J` (erase from cursor to end of screen).

/// How many lines the last-written frame occupied. `0` means no frame has
/// been written yet, so the next call writes with no leading erase sequence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RedrawState {
    previous_lines: usize,
}

impl RedrawState {
    /// A fresh state: no prior frame.
    pub fn new() -> Self {
        RedrawState::default()
    }

    /// Build the bytes to write for the next frame: an erase sequence for
    /// the previous frame (skipped on the first call), followed by `text`.
    /// Updates the tracked line count to `line_count` for the following
    /// call.
    pub fn frame(&mut self, text: &str, line_count: usize) -> String {
        let mut out = String::new();
        if self.previous_lines > 0 {
            out.push_str(&format!("\x1b[{}A", self.previous_lines));
            out.push_str("\x1b[0J");
        }
        out.push_str(text);
        self.previous_lines = line_count;
        out
    }

    /// Clear any outstanding frame without writing a new one, so a redraw
    /// never interleaves with what is printed next (FR-012): the completion
    /// summary, or the run ending on a non-terminal path having never drawn
    /// anything.
    pub fn clear(&mut self) -> String {
        if self.previous_lines == 0 {
            return String::new();
        }
        let out = format!("\x1b[{}A\x1b[0J", self.previous_lines);
        self.previous_lines = 0;
        out
    }

    /// Forget the previous frame without erasing it: the caller has already
    /// written something else (an ordinary progress line) below it, so the
    /// old frame is now inert scrollback rather than a block this state can
    /// safely erase (Codex review of PR #196). Erasing anyway would use the
    /// stale line count against a cursor position that has moved since the
    /// frame was drawn, wiping the just-written progress line along with
    /// part of the old frame rather than either cleanly. The next call to
    /// [`RedrawState::frame`] then draws fresh, with no erase prefix,
    /// landing correctly below whatever was just printed.
    pub fn forget(&mut self) {
        self.previous_lines = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_frame_writes_no_erase_sequence() {
        let mut state = RedrawState::new();
        let out = state.frame("hello\n", 1);
        assert_eq!(out, "hello\n");
    }

    #[test]
    fn a_second_frame_erases_the_first_frames_line_count_first() {
        let mut state = RedrawState::new();
        state.frame("a\nb\nc\n", 3);
        let out = state.frame("x\n", 1);
        assert_eq!(out, "\x1b[3A\x1b[0Jx\n");
    }

    #[test]
    fn the_state_tracks_the_newest_frames_line_count_for_the_next_call() {
        let mut state = RedrawState::new();
        state.frame("a\nb\n", 2);
        state.frame("x\n", 1);
        let out = state.frame("y\n", 1);
        assert_eq!(out, "\x1b[1A\x1b[0Jy\n");
    }

    #[test]
    fn clearing_with_no_prior_frame_writes_nothing() {
        let mut state = RedrawState::new();
        assert_eq!(state.clear(), "");
    }

    #[test]
    fn clearing_erases_the_last_frame_and_resets_the_tracked_count() {
        let mut state = RedrawState::new();
        state.frame("a\nb\n", 2);
        assert_eq!(state.clear(), "\x1b[2A\x1b[0J");
        // A frame drawn after a clear starts fresh, with no further erase.
        let out = state.frame("z\n", 1);
        assert_eq!(out, "z\n");
    }

    #[test]
    fn forgetting_writes_nothing_and_the_next_frame_has_no_erase_prefix() {
        let mut state = RedrawState::new();
        state.frame("a\nb\nc\n", 3);
        state.forget();
        // Unlike `clear`, `forget` returns nothing: the caller already wrote
        // something else (an ordinary progress line) and must not have this
        // state emit an erase sequence against a cursor position that has
        // since moved.
        let out = state.frame("fresh\n", 1);
        assert_eq!(out, "fresh\n");
    }
}
