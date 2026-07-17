//! Event model for the live agent dashboard.
//!
//! The dashboard is intentionally decoupled from any particular event source.
//! It consumes a stream of [`AgentEvent`]s over an [`std::sync::mpsc`] channel,
//! so the same UI can be driven by the demo producer (see [`super::demo`]) today
//! and by the real agent loop later — the agent simply emits the same events as
//! it analyzes the AST, greps the workspace, reads files and streams a diff.

/// Lifecycle of a file as the agent works through the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    /// Known to exist but not yet touched this turn.
    Pending,
    /// Currently being scanned / grepped.
    Scanning,
    /// Has been read into context.
    Read,
    /// Has been (or is being) edited by the agent.
    Modified,
}

impl FileStatus {
    /// Single-character glyph shown next to the file in the tree.
    pub fn glyph(self) -> char {
        match self {
            FileStatus::Pending => '·',
            FileStatus::Scanning => '◐',
            FileStatus::Read => '○',
            FileStatus::Modified => '●',
        }
    }

    /// Short human label used in the legend / tooltips.
    pub fn label(self) -> &'static str {
        match self {
            FileStatus::Pending => "pending",
            FileStatus::Scanning => "scanning",
            FileStatus::Read => "read",
            FileStatus::Modified => "modified",
        }
    }
}

/// The kind of a single line in the streaming diff view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffKind {
    /// Unchanged context line.
    Context,
    /// Line being added (rendered green, prefixed with `+`).
    Added,
    /// Line being removed (rendered red, prefixed with `-`).
    Removed,
    /// A hunk header such as `@@ -1,7 +1,9 @@`.
    Hunk,
}

/// A single step in the agent's reasoning/tool pipeline (left panel).
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Short stage tag, e.g. `Analyzing AST`, `Running Grep`, `Generating Diff`.
    pub stage: String,
    /// Free-form detail for the stage.
    pub detail: String,
}

/// Everything the dashboard knows how to render, as a stream of events.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// Append a step to the agent pipeline log.
    Log(LogEntry),
    /// Update the overall single-line status (header).
    Status(String),
    /// Set the progress bar to a value in `0..=100`.
    Progress(u16),
    /// Register or update a file's status in the tree.
    File { path: String, status: FileStatus },
    /// Begin streaming a fresh diff for `file` (clears the previous diff).
    DiffBegin { file: String, language: String },
    /// Start a new diff line of the given kind (cursor moves to a fresh line).
    DiffNewLine(DiffKind),
    /// Append a single character to the current diff line.
    DiffPush(char),
    /// The agent finished its turn; the UI stays up until the user quits.
    Done,
}

#[cfg(test)]
mod tests {
    use super::{AgentEvent, DiffKind, FileStatus, LogEntry};

    #[test]
    fn file_status_glyphs_are_distinct() {
        let glyphs: Vec<char> = [
            FileStatus::Pending,
            FileStatus::Scanning,
            FileStatus::Read,
            FileStatus::Modified,
        ]
        .iter()
        .map(|s| s.glyph())
        .collect();
        let unique: std::collections::HashSet<_> = glyphs.iter().copied().collect();
        assert_eq!(
            unique.len(),
            glyphs.len(),
            "each FileStatus must have a unique glyph"
        );
    }

    #[test]
    fn file_status_labels_are_nonempty() {
        for status in [
            FileStatus::Pending,
            FileStatus::Scanning,
            FileStatus::Read,
            FileStatus::Modified,
        ] {
            assert!(
                !status.label().is_empty(),
                "{status:?} label must not be empty"
            );
        }
    }

    #[test]
    fn file_status_progression_variants_exist() {
        // All four lifecycle stages must be constructible.
        let _: [FileStatus; 4] = [
            FileStatus::Pending,
            FileStatus::Scanning,
            FileStatus::Read,
            FileStatus::Modified,
        ];
    }

    #[test]
    fn diff_kind_variants_exist() {
        let _: [DiffKind; 4] = [
            DiffKind::Context,
            DiffKind::Added,
            DiffKind::Removed,
            DiffKind::Hunk,
        ];
    }

    #[test]
    fn agent_event_log_holds_stage_and_detail() {
        let entry = LogEntry {
            stage: "Analyzing AST".to_string(),
            detail: "ok".to_string(),
        };
        let event = AgentEvent::Log(entry.clone());
        assert!(matches!(event, AgentEvent::Log(e) if e.stage == "Analyzing AST"));
    }

    #[test]
    fn agent_event_progress_clamps_semantics() {
        // Progress is a u16; values above 100 are accepted by the enum but clamped
        // in the App state machine. Here we just verify the variant is constructible.
        let event = AgentEvent::Progress(200);
        assert!(matches!(event, AgentEvent::Progress(200)));
    }

    #[test]
    fn agent_event_file_captures_path_and_status() {
        let event = AgentEvent::File {
            path: "src/main.rs".to_string(),
            status: FileStatus::Modified,
        };
        assert!(
            matches!(event, AgentEvent::File { ref path, status: FileStatus::Modified } if path == "src/main.rs")
        );
    }

    #[test]
    fn agent_event_diff_begin_captures_file_and_language() {
        let event = AgentEvent::DiffBegin {
            file: "lib.rs".to_string(),
            language: "rust".to_string(),
        };
        assert!(
            matches!(event, AgentEvent::DiffBegin { ref file, ref language } if file == "lib.rs" && language == "rust")
        );
    }

    #[test]
    fn agent_event_done_is_unit_variant() {
        let event = AgentEvent::Done;
        assert!(matches!(event, AgentEvent::Done));
    }
}
