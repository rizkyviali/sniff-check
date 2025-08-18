// Unified progress tracking utilities

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Builder for creating consistent progress bars across the application
pub struct ProgressBarBuilder {
    quiet: bool,
    message: String,
    length: Option<u64>,
}

impl ProgressBarBuilder {
    /// Create a new progress bar builder
    pub fn new() -> Self {
        Self {
            quiet: false,
            message: "Processing...".to_string(),
            length: None,
        }
    }

    /// Set whether to show progress (false when quiet=true)
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    /// Set the progress message
    pub fn message<S: Into<String>>(mut self, message: S) -> Self {
        self.message = message.into();
        self
    }

    /// Set the length for determinate progress bars
    pub fn length(mut self, length: u64) -> Self {
        self.length = Some(length);
        self
    }

    /// Build a spinner (indeterminate progress)
    pub fn spinner(self) -> Option<ProgressBar> {
        if self.quiet {
            return None;
        }

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(self.message);
        pb.enable_steady_tick(Duration::from_millis(80));
        Some(pb)
    }

    /// Build a progress bar (determinate progress)
    pub fn progress_bar(self) -> Option<ProgressBar> {
        if self.quiet {
            return None;
        }

        let length = self.length.unwrap_or(100);
        let pb = ProgressBar::new(length);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(self.message);
        Some(pb)
    }
}

/// Unified progress tracker for file operations
pub struct FileProgressTracker {
    progress_bar: Option<ProgressBar>,
    start_time: std::time::Instant,
    min_display_time: Duration,
}

impl FileProgressTracker {
    /// Create a new file progress tracker
    pub fn new(message: &str, total_files: Option<usize>, quiet: bool) -> Self {
        let progress_bar = if let Some(total) = total_files {
            ProgressBarBuilder::new()
                .quiet(quiet)
                .message(message)
                .length(total as u64)
                .progress_bar()
        } else {
            ProgressBarBuilder::new()
                .quiet(quiet)
                .message(message)
                .spinner()
        };

        Self {
            progress_bar,
            start_time: std::time::Instant::now(),
            min_display_time: Duration::from_millis(200),
        }
    }

    /// Increment progress by 1
    pub fn inc(&self) {
        if let Some(pb) = &self.progress_bar {
            pb.inc(1);
        }
    }

    /// Set progress to a specific value
    pub fn set_position(&self, pos: u64) {
        if let Some(pb) = &self.progress_bar {
            pb.set_position(pos);
        }
    }

    /// Update the progress message
    pub fn set_message(&self, message: &str) {
        if let Some(pb) = &self.progress_bar {
            pb.set_message(message.to_string());
        }
    }

    /// Finish the progress bar with a completion message
    pub fn finish_with_message(&self, message: &str) {
        // Ensure minimum display time for visibility
        let elapsed = self.start_time.elapsed();
        if elapsed < self.min_display_time {
            std::thread::sleep(self.min_display_time - elapsed);
        }

        if let Some(pb) = &self.progress_bar {
            pb.finish_with_message(message.to_string());
        }
    }

    /// Finish the progress bar and clear it
    pub fn finish_and_clear(&self) {
        let elapsed = self.start_time.elapsed();
        if elapsed < self.min_display_time {
            std::thread::sleep(self.min_display_time - elapsed);
        }

        if let Some(pb) = &self.progress_bar {
            pb.finish_and_clear();
        }
    }
}

impl Drop for FileProgressTracker {
    fn drop(&mut self) {
        if let Some(pb) = &self.progress_bar {
            if !pb.is_finished() {
                pb.finish_and_clear();
            }
        }
    }
}

/// Convenience function for creating a spinner with default settings
pub fn create_spinner(message: &str, quiet: bool) -> Option<ProgressBar> {
    ProgressBarBuilder::new()
        .quiet(quiet)
        .message(message)
        .spinner()
}

/// Convenience function for creating a progress bar with default settings
pub fn create_progress_bar(message: &str, total: u64, quiet: bool) -> Option<ProgressBar> {
    ProgressBarBuilder::new()
        .quiet(quiet)
        .message(message)
        .length(total)
        .progress_bar()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_builder() {
        // Test spinner creation (quiet mode)
        let spinner = ProgressBarBuilder::new()
            .quiet(true)
            .message("Testing")
            .spinner();
        assert!(spinner.is_none());

        // Test progress bar creation (quiet mode)
        let progress = ProgressBarBuilder::new()
            .quiet(true)
            .message("Testing")
            .length(100)
            .progress_bar();
        assert!(progress.is_none());
    }

    #[test]
    fn test_file_progress_tracker() {
        let tracker = FileProgressTracker::new("Testing", Some(10), true);
        tracker.inc();
        tracker.set_position(5);
        tracker.set_message("Updated message");
        tracker.finish_with_message("Done");
        // Should not panic when dropped
    }
}