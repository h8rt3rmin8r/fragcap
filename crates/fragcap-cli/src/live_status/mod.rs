// SPDX-License-Identifier: Apache-2.0

//! The live capture status display (slice S069, issue #186).
//!
//! [`LiveStatusSnapshot`] is a plain, platform-independent value assembled
//! once per tick from the handles `drive_live` already holds (`GateHandle`,
//! the new `fragcap::LiveStats`, the stamper's active endpoint count, the
//! elapsed clock, and the currently bound process). [`render_status`] is a
//! pure function over that snapshot, so every rendering rule is unit-tested
//! on any platform, even though the one call site that constructs a
//! snapshot from a live run is gated to Windows/ETW (research R-5: cfg-gate
//! the data source, not the logic).

pub mod heartbeat;
pub mod redraw;

use std::sync::Arc;
use std::time::Duration;

use crate::color::{use_color, Stream, RESET, WARN};

/// The bound process a status block's header line names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundProcess {
    /// `None` when the pid is bound but no `Started` event has yet recorded
    /// its image name (the attach-to-running seeding path, or a narrow
    /// timing window right at a fresh match): distinct from an empty
    /// string, so the renderer can say so rather than leaving a blank gap
    /// (Copilot review of PR #196).
    pub name: Option<String>,
    pub pid: u32,
    pub role: String,
    /// The stage a role binds as, when the session tracked one distinct from
    /// the role itself (`crate::session::role_bindings`'s third element).
    pub stage: Option<String>,
}

/// The top holder-tally entries a status block shows before truncating.
const HOLDER_ROWS: usize = 5;

/// A point-in-time read of everything the live status block or the optional
/// `capture.progress` JSON event needs. Carries no reference, no lock guard,
/// and no platform type, so it is cheap to build in a test and cheap to move
/// across the one tick that constructs it in `drive_live`.
#[derive(Clone, Debug, PartialEq)]
pub struct LiveStatusSnapshot {
    pub elapsed: Duration,
    pub process: Option<BoundProcess>,
    pub written_packets: u64,
    pub written_bytes: u64,
    pub byte_bound: Option<u64>,
    pub packet_bound: Option<u64>,
    pub active_endpoints: usize,
    pub narrowed: bool,
    pub watch_discarded: u64,
    pub out_of_window_discarded: u64,
    pub scope_discarded: u64,
    pub scope_unresolved_discarded: u64,
    pub buffer_dropped: u64,
    pub sink_dropped: u64,
    /// Sorted by count descending then name ascending, matching
    /// `fragcap::LiveStats::holder_tally_snapshot`'s own tiebreak.
    pub holder_tally: Vec<(Arc<str>, u64)>,
}

/// Render the status block, following `contracts/status-block.md`. Returns
/// the frame's text and its line count, since [`redraw::RedrawState`] needs
/// the count to erase the right number of lines before the next frame.
///
/// Values hang beneath measured four-space anchors. Exact tokens may exceed
/// the width; complete ANSI sequences remain intact and redraw counts physical
/// rows occupied by terminal wrapping when a terminal width is available.
pub fn render_status(
    snapshot: &LiveStatusSnapshot,
    use_color_flag: bool,
    width: Option<usize>,
) -> (String, usize) {
    let terminal_width = width;
    let width = width.unwrap_or(80);
    let process = snapshot
        .process
        .as_ref()
        .map(|p| {
            let name =
                crate::display::human_display_value(p.name.as_deref().unwrap_or("(name unknown)"));
            match p.stage.as_deref() {
                Some(stage) if stage != p.role => {
                    format!("capturing {name} pid {} role {}/{}", p.pid, p.role, stage)
                }
                _ => format!("capturing {name} pid {} role {}", p.pid, p.role),
            }
        })
        .unwrap_or_else(|| "waiting for a target".into());
    let secs = snapshot.elapsed.as_secs();
    let elapsed = format!(
        "{:02}:{:02}:{:02}",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    );
    let volume = match (snapshot.byte_bound, snapshot.packet_bound) {
        (Some(bound), _) => format!("{} / {bound} bytes", snapshot.written_bytes),
        (None, Some(bound)) => format!("{} / {bound} pkts", snapshot.written_packets),
        _ => format!(
            "{} pkts, {} bytes",
            snapshot.written_packets, snapshot.written_bytes
        ),
    };
    let fields = vec![
        ("fragcap".into(), process),
        ("elapsed".into(), elapsed),
        ("written".into(), volume),
        (
            "filter".into(),
            if snapshot.narrowed {
                format!("narrowed, {} endpoint(s)", snapshot.active_endpoints)
            } else {
                "not yet narrowed".into()
            },
        ),
    ];
    let mut text = crate::display::render_fields(2, &fields, width);
    let rows: Vec<Vec<String>> = [
        ("watch", snapshot.watch_discarded),
        ("window", snapshot.out_of_window_discarded),
        ("scope", snapshot.scope_discarded),
        ("unresolved", snapshot.scope_unresolved_discarded),
        ("buffer", snapshot.buffer_dropped),
        ("sink", snapshot.sink_dropped),
    ]
    .into_iter()
    .map(|(label, count)| {
        vec![
            label.into(),
            if use_color_flag && count > 0 {
                format!("{WARN}{count}{RESET}")
            } else {
                count.to_string()
            },
        ]
    })
    .collect();
    text.push_str("discards\n");
    let layout = crate::display::ColumnLayout::new(2, &rows);
    for row in rows {
        text.push_str(&layout.render_wrapped_row(&row, width));
        text.push('\n');
    }
    let fields: Vec<(String, String)> = snapshot
        .holder_tally
        .iter()
        .take(HOLDER_ROWS)
        .map(|(image, count)| (image.to_string(), count.to_string()))
        .collect();
    text.push_str(&crate::display::render_fields(4, &fields, width));
    if snapshot.holder_tally.len() > HOLDER_ROWS {
        text.push_str(&format!(
            "    ... and {} more\n",
            snapshot.holder_tally.len() - HOLDER_ROWS
        ));
    }
    let count = text
        .lines()
        .map(|line| {
            terminal_width.map_or(1, |width| {
                crate::display::display_width(line)
                    .div_ceil(width.max(1))
                    .max(1)
            })
        })
        .sum();
    (text, count)
}
/// Whether the live status display should render on this run: stderr is a
/// real terminal, `NO_COLOR`-independent (color and terminal-ness are
/// separate questions; `use_color` folds `NO_COLOR` in only for the color
/// decision, not for whether a redraw happens at all).
pub fn is_terminal() -> bool {
    use std::io::IsTerminal;
    std::io::stderr().is_terminal()
}

/// Whether to colorize the status block: stderr is a terminal and
/// `NO_COLOR` is unset (FR-010).
pub fn use_status_color() -> bool {
    use_color(Stream::Stderr)
}

/// Standard error's current column width, when it can be determined.
///
/// Queried fresh on every redraw so wrapping and occupied-row accounting follow
/// terminal resizes. `None` selects the default report width when the terminal
/// cannot report one. This is called only from the terminal-gated redraw path.
pub fn terminal_width() -> Option<usize> {
    terminal_size::terminal_size_of(std::io::stderr()).map(|(width, _)| width.0 as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_snapshot() -> LiveStatusSnapshot {
        LiveStatusSnapshot {
            elapsed: Duration::from_secs(0),
            process: None,
            written_packets: 0,
            written_bytes: 0,
            byte_bound: None,
            packet_bound: None,
            active_endpoints: 0,
            narrowed: false,
            watch_discarded: 0,
            out_of_window_discarded: 0,
            scope_discarded: 0,
            scope_unresolved_discarded: 0,
            buffer_dropped: 0,
            sink_dropped: 0,
            holder_tally: Vec::new(),
        }
    }

    #[test]
    fn a_snapshot_with_no_process_renders_a_waiting_header_rather_than_panicking() {
        let (text, lines) = render_status(&base_snapshot(), false, None);
        assert!(text.contains("waiting for a target"));
        assert!(lines >= 4, "header, elapsed, filter, discards at least");
    }

    #[test]
    fn a_configured_bound_renders_written_against_it_and_an_unbounded_run_does_not() {
        let mut with_bound = base_snapshot();
        with_bound.written_bytes = 100;
        with_bound.byte_bound = Some(1000);
        let (text, _) = render_status(&with_bound, false, None);
        assert!(text.contains("100 / 1000 bytes"));

        let mut no_bound = base_snapshot();
        no_bound.written_packets = 42;
        no_bound.written_bytes = 999;
        let (text, _) = render_status(&no_bound, false, None);
        assert!(text.contains("42 pkts, 999 bytes"));
        assert!(
            !text.contains('/'),
            "no bound comparison when none is configured"
        );
    }

    #[test]
    fn narrowing_state_is_always_explicit() {
        let mut not_narrowed = base_snapshot();
        not_narrowed.narrowed = false;
        let (text, _) = render_status(&not_narrowed, false, None);
        assert!(text.contains("not yet narrowed"));

        let mut narrowed = base_snapshot();
        narrowed.narrowed = true;
        narrowed.active_endpoints = 3;
        let (text, _) = render_status(&narrowed, false, None);
        assert!(text.contains("narrowed, 3 endpoint(s)"));
    }

    #[test]
    fn every_discard_counter_appears_and_a_nonzero_one_colors_when_asked() {
        let mut s = base_snapshot();
        s.watch_discarded = 1;
        s.out_of_window_discarded = 2;
        s.scope_discarded = 3;
        s.scope_unresolved_discarded = 4;
        s.buffer_dropped = 5;
        s.sink_dropped = 6;

        let (plain, _) = render_status(&s, false, None);
        for (label, value) in [
            ("watch", 1),
            ("window", 2),
            ("scope", 3),
            ("unresolved", 4),
            ("buffer", 5),
            ("sink", 6),
        ] {
            assert!(plain
                .lines()
                .any(|line| line.trim_start().starts_with(label)
                    && line.trim_end().ends_with(&value.to_string())));
        }
        assert!(!plain.contains(WARN), "no color when use_color is false");

        let (colored, _) = render_status(&s, true, None);
        assert!(colored.contains(WARN));
        assert!(colored.contains(RESET));

        let mut zero = base_snapshot();
        zero.watch_discarded = 0;
        let (zero_colored, _) = render_status(&zero, true, None);
        assert!(
            !zero_colored.contains(WARN),
            "a zero counter never colors, even with use_color true"
        );
    }

    #[test]
    fn the_holder_tally_shows_the_top_five_and_a_trailing_overflow_count() {
        let mut few = base_snapshot();
        few.holder_tally = vec![
            (Arc::from("a.exe"), 3),
            (Arc::from("b.exe"), 2),
            (Arc::from("c.exe"), 1),
        ];
        let (text, _) = render_status(&few, false, None);
        assert!(text.contains("a.exe"));
        assert!(text.contains("b.exe"));
        assert!(text.contains("c.exe"));
        assert!(!text.contains("more"));

        let mut many = base_snapshot();
        many.holder_tally = (0..8)
            .map(|i| (Arc::from(format!("p{i}.exe")) as Arc<str>, (8 - i) as u64))
            .collect();
        let (text, _) = render_status(&many, false, None);
        for i in 0..5 {
            assert!(text.contains(&format!("p{i}.exe")));
        }
        assert!(!text.contains("p5.exe"));
        assert!(text.contains("... and 3 more"));
    }

    #[test]
    fn a_narrow_width_preserves_exact_values_and_complete_color_sequences() {
        let mut s = base_snapshot();
        s.sink_dropped = 9;
        s.process = Some(BoundProcess {
            name: Some("AngelLegion.exe".to_string()),
            pid: 44460,
            role: "target".to_string(),
            stage: Some("client".to_string()),
        });

        let (wide, _) = render_status(&s, true, None);
        assert!(wide.contains(WARN), "color applies with no width limit");

        let (narrow, physical_rows) = render_status(&s, true, Some(10));
        assert!(narrow.contains("AngelLegion.exe"));
        assert!(narrow.contains("44460"));
        assert!(narrow.contains("client"));
        assert!(narrow.contains(WARN) && narrow.contains(RESET));
        assert!(!narrow.contains("AngelLegio\n"));
        assert!(physical_rows > narrow.lines().count());
    }

    // S069 T039, SC-005. Reproduces the run that prompted issue #186: a
    // background process dominates the file (measured at 91 percent) while
    // the target itself contributes only a sliver. The first rendered frame
    // after admission begins must already name the dominant contributor,
    // not only the end-of-run summary.
    #[test]
    fn a_dominant_non_target_contributor_appears_in_the_first_rendered_frame() {
        let mut s = base_snapshot();
        s.process = Some(BoundProcess {
            name: Some("AngelLegion.exe".to_string()),
            pid: 44460,
            role: "target".to_string(),
            stage: None,
        });
        s.written_packets = 1000;
        s.holder_tally = vec![
            (Arc::from("com.docker.backend.exe"), 910),
            (Arc::from("AngelLegion.exe"), 90),
        ];

        let (text, _) = render_status(&s, false, None);
        assert!(
            text.contains("com.docker.backend.exe"),
            "the dominant non-target contributor must be visible in the very \
             first frame, not only after the run ends"
        );
        // The dominant-first order (`LiveStats::holder_tally_snapshot`'s own
        // count-descending tiebreak discipline, reflected here in the
        // snapshot's pre-sorted input) puts the 910-count holder row ahead
        // of the 90-count target holder row, so a reader scanning top to
        // bottom sees the surprising fact first, not buried below its own
        // target.
        // Holder rows are the only lines with `render_status`'s 4-space
        // indent (`contracts/status-block.md`'s process breakdown section);
        // the header line uses a 2-space indent, so this excludes it even
        // though the header also names the target by its own image name.
        let holder_rows: Vec<&str> = text.lines().filter(|l| l.starts_with("    ")).collect();
        let docker_row = holder_rows
            .iter()
            .position(|l| l.contains("com.docker.backend.exe"))
            .expect("a docker holder row exists");
        let target_row = holder_rows
            .iter()
            .position(|l| l.contains("AngelLegion.exe"))
            .expect("a target holder row exists");
        assert!(
            docker_row < target_row,
            "the dominant contributor's holder row must precede the target's own holder row: {holder_rows:?}"
        );
    }
}
