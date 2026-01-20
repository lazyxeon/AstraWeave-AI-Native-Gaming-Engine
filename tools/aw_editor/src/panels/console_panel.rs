use super::Panel;
use egui::Ui;
use std::collections::VecDeque;

/// Log entry with full metadata
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub message: String,
    pub level: LogLevel,
    pub timestamp: std::time::SystemTime,
    pub category: Option<String>,
    pub source_file: Option<String>,
    pub source_line: Option<u32>,
    pub stacktrace: Option<String>,
}

impl LogEntry {
    /// Create a new log entry with current timestamp
    pub fn new(message: impl Into<String>, level: LogLevel) -> Self {
        Self {
            message: message.into(),
            level,
            timestamp: std::time::SystemTime::now(),
            category: None,
            source_file: None,
            source_line: None,
            stacktrace: None,
        }
    }

    /// Create a log entry with category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Create a log entry with source location
    pub fn with_source(mut self, file: impl Into<String>, line: u32) -> Self {
        self.source_file = Some(file.into());
        self.source_line = Some(line);
        self
    }

    /// Create a log entry with stacktrace
    pub fn with_stacktrace(mut self, trace: impl Into<String>) -> Self {
        self.stacktrace = Some(trace.into());
        self
    }

    /// Format timestamp for display
    pub fn format_timestamp(&self) -> String {
        use std::time::UNIX_EPOCH;
        let duration = self.timestamp.duration_since(UNIX_EPOCH).unwrap_or_default();
        let secs = duration.as_secs();
        let hours = (secs / 3600) % 24;
        let minutes = (secs / 60) % 60;
        let seconds = secs % 60;
        let millis = duration.subsec_millis();
        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum LogLevel {
    Debug,
    #[default]
    Info,
    Warning,
    Error,
    All,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

impl LogLevel {
    pub fn all_levels() -> &'static [LogLevel] {
        &[LogLevel::All, LogLevel::Debug, LogLevel::Info, LogLevel::Warning, LogLevel::Error]
    }

    pub fn matches(&self, log: &str) -> bool {
        match self {
            LogLevel::All => true,
            LogLevel::Debug => log.contains("🔍") || log.to_lowercase().contains("[debug]"),
            LogLevel::Info => !log.contains("⚠️") && !log.contains("❌") && !log.contains("🔍"),
            LogLevel::Warning => log.contains("⚠️"),
            LogLevel::Error => log.contains("❌"),
        }
    }

    pub fn matches_entry(&self, entry: &LogEntry) -> bool {
        match self {
            LogLevel::All => true,
            _ => entry.level == *self,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LogLevel::All => "All",
            LogLevel::Debug => "Debug",
            LogLevel::Info => "Info",
            LogLevel::Warning => "Warnings",
            LogLevel::Error => "Errors",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            LogLevel::All => "📋",
            LogLevel::Debug => "🔍",
            LogLevel::Info => "ℹ️",
            LogLevel::Warning => "⚠️",
            LogLevel::Error => "❌",
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            LogLevel::All => egui::Color32::LIGHT_GRAY,
            LogLevel::Debug => egui::Color32::from_rgb(150, 150, 255),
            LogLevel::Info => egui::Color32::from_rgb(100, 149, 237),
            LogLevel::Warning => egui::Color32::from_rgb(255, 200, 100),
            LogLevel::Error => egui::Color32::from_rgb(255, 100, 100),
        }
    }
}

/// Console panel with enhanced logging features
pub struct ConsolePanel {
    filter_level: LogLevel,
    search_text: String,
    auto_scroll: bool,
    show_timestamps: bool,
    show_categories: bool,
    show_source_location: bool,
    
    // Enhanced log storage
    log_entries: VecDeque<LogEntry>,
    max_entries: usize,
    
    // Category filtering
    enabled_categories: std::collections::HashSet<String>,
    all_categories: Vec<String>,
    
    // Expanded stacktraces (by index)
    expanded_stacktraces: std::collections::HashSet<usize>,
    
    // Console command input
    command_input: String,
    command_history: VecDeque<String>,
    command_history_index: Option<usize>,
    
    // Pause/resume
    is_paused: bool,
    paused_entry_count: usize,
}

impl ConsolePanel {
    pub fn new() -> Self {
        Self {
            filter_level: LogLevel::All,
            search_text: String::new(),
            auto_scroll: true,
            show_timestamps: true,
            show_categories: true,
            show_source_location: false,
            log_entries: VecDeque::with_capacity(1000),
            max_entries: 1000,
            enabled_categories: std::collections::HashSet::new(),
            all_categories: Vec::new(),
            expanded_stacktraces: std::collections::HashSet::new(),
            command_input: String::new(),
            command_history: VecDeque::with_capacity(50),
            command_history_index: None,
            is_paused: false,
            paused_entry_count: 0,
        }
    }

    /// Push a new log entry
    pub fn push_entry(&mut self, entry: LogEntry) {
        if self.is_paused {
            self.paused_entry_count += 1;
            return;
        }
        
        // Track category
        if let Some(cat) = &entry.category {
            if !self.all_categories.contains(cat) {
                self.all_categories.push(cat.clone());
            }
            if self.enabled_categories.is_empty() {
                // First entry - enable all categories by default
                self.enabled_categories.insert(cat.clone());
            }
        }
        
        // Enforce max entries
        if self.log_entries.len() >= self.max_entries {
            self.log_entries.pop_front();
        }
        self.log_entries.push_back(entry);
    }

    /// Push a simple log message (backwards compatible)
    pub fn push_log(&mut self, message: impl Into<String>, level: LogLevel) {
        self.push_entry(LogEntry::new(message, level));
    }

    /// Clear all logs
    pub fn clear(&mut self) {
        self.log_entries.clear();
        self.expanded_stacktraces.clear();
        self.paused_entry_count = 0;
    }

    /// Get entry counts by level
    pub fn get_counts(&self) -> (usize, usize, usize, usize, usize) {
        let total = self.log_entries.len();
        let mut debug = 0;
        let mut info = 0;
        let mut warnings = 0;
        let mut errors = 0;
        
        for entry in &self.log_entries {
            match entry.level {
                LogLevel::Debug => debug += 1,
                LogLevel::Info => info += 1,
                LogLevel::Warning => warnings += 1,
                LogLevel::Error => errors += 1,
                LogLevel::All => {}
            }
        }
        (total, debug, info, warnings, errors)
    }

    /// Legacy counting for string-based logs
    pub fn get_counts_legacy(logs: &[String]) -> (usize, usize, usize, usize) {
        let total = logs.len();
        let warnings = logs.iter().filter(|l| l.contains("⚠️")).count();
        let errors = logs.iter().filter(|l| l.contains("❌")).count();
        let infos = total - warnings - errors;
        (total, infos, warnings, errors)
    }

    pub fn filter_logs<'a>(&self, logs: &'a [String]) -> Vec<&'a String> {
        logs.iter()
            .filter(|log| {
                self.filter_level.matches(log)
                    && (self.search_text.is_empty()
                        || log
                            .to_lowercase()
                            .contains(&self.search_text.to_lowercase()))
            })
            .collect()
    }

    /// Filter log entries
    fn filter_entries(&self) -> Vec<(usize, &LogEntry)> {
        self.log_entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                // Level filter
                if !self.filter_level.matches_entry(entry) {
                    return false;
                }
                
                // Category filter
                if let Some(cat) = &entry.category {
                    if !self.enabled_categories.is_empty() && !self.enabled_categories.contains(cat) {
                        return false;
                    }
                }
                
                // Text search
                if !self.search_text.is_empty() {
                    let search_lower = self.search_text.to_lowercase();
                    if !entry.message.to_lowercase().contains(&search_lower) {
                        return false;
                    }
                }
                
                true
            })
            .collect()
    }

    /// Execute a console command
    fn execute_command(&mut self, command: &str) {
        let cmd = command.trim();
        if cmd.is_empty() {
            return;
        }

        // Add to history
        self.command_history.push_front(cmd.to_string());
        if self.command_history.len() > 50 {
            self.command_history.pop_back();
        }
        self.command_history_index = None;

        // Parse and execute
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.first() {
            Some(&"clear") => self.clear(),
            Some(&"help") => {
                self.push_entry(LogEntry::new(
                    "Available commands: clear, help, pause, resume, filter <level>",
                    LogLevel::Info,
                ).with_category("Console"));
            }
            Some(&"pause") => {
                self.is_paused = true;
                self.push_entry(LogEntry::new("Logging paused", LogLevel::Info).with_category("Console"));
            }
            Some(&"resume") => {
                self.is_paused = false;
                self.push_entry(LogEntry::new(
                    format!("Logging resumed ({} entries skipped)", self.paused_entry_count),
                    LogLevel::Info,
                ).with_category("Console"));
                self.paused_entry_count = 0;
            }
            Some(&"filter") => {
                if let Some(&level) = parts.get(1) {
                    match level.to_lowercase().as_str() {
                        "all" => self.filter_level = LogLevel::All,
                        "debug" => self.filter_level = LogLevel::Debug,
                        "info" => self.filter_level = LogLevel::Info,
                        "warning" | "warn" => self.filter_level = LogLevel::Warning,
                        "error" => self.filter_level = LogLevel::Error,
                        _ => {
                            self.push_entry(LogEntry::new(
                                format!("Unknown filter level: {}", level),
                                LogLevel::Warning,
                            ).with_category("Console"));
                        }
                    }
                }
            }
            Some(unknown) => {
                self.push_entry(LogEntry::new(
                    format!("Unknown command: {}. Type 'help' for available commands.", unknown),
                    LogLevel::Warning,
                ).with_category("Console"));
            }
            None => {}
        }
    }

    /// Export logs to string
    pub fn export_logs(&self) -> String {
        let mut output = String::new();
        for entry in &self.log_entries {
            let timestamp = entry.format_timestamp();
            let level = entry.level.name();
            let category = entry.category.as_deref().unwrap_or("-");
            output.push_str(&format!("[{}] [{}] [{}] {}\n", timestamp, level, category, entry.message));
            if let Some(trace) = &entry.stacktrace {
                output.push_str(&format!("  Stacktrace:\n{}\n", trace));
            }
        }
        output
    }

    pub fn show_with_logs(&mut self, ui: &mut Ui, logs: &mut Vec<String>) -> bool {
        let mut cleared = false;

        let (total_count, info_count, warning_count, error_count) = Self::get_counts_legacy(logs);

        ui.horizontal(|ui| {
            ui.heading("Console");
            ui.separator();
            ui.label(format!("{} logs", total_count));

            if info_count > 0 {
                ui.label(
                    egui::RichText::new(format!("{} info", info_count))
                        .color(egui::Color32::from_rgb(100, 149, 237)),
                );
            }
            if warning_count > 0 {
                ui.label(
                    egui::RichText::new(format!("{} warn", warning_count))
                        .color(egui::Color32::YELLOW),
                );
            }
            if error_count > 0 {
                ui.label(
                    egui::RichText::new(format!("{} err", error_count)).color(egui::Color32::RED),
                );
            }
        });

        ui.separator();

        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("log_level_filter")
                .selected_text(self.filter_level.name())
                .width(90.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter_level, LogLevel::All, "All");
                    ui.selectable_value(&mut self.filter_level, LogLevel::Info, "Info");
                    ui.selectable_value(&mut self.filter_level, LogLevel::Warning, "Warnings");
                    ui.selectable_value(&mut self.filter_level, LogLevel::Error, "Errors");
                });

            ui.add(
                egui::TextEdit::singleline(&mut self.search_text)
                    .hint_text("Search logs...")
                    .desired_width(150.0),
            );

            if ui.small_button("X").on_hover_text("Clear search").clicked() {
                self.search_text.clear();
            }

            ui.separator();

            ui.checkbox(&mut self.auto_scroll, "Auto-scroll");

            if ui.button("Clear").on_hover_text("Clear all logs").clicked() {
                logs.clear();
                cleared = true;
            }

            if ui
                .button("Copy All")
                .on_hover_text("Copy logs to clipboard")
                .clicked()
            {
                let all_logs = logs.join("\n");
                ui.ctx().copy_text(all_logs);
            }
        });

        ui.add_space(4.0);

        let filtered_logs = self.filter_logs(logs);

        let scroll_area = egui::ScrollArea::vertical()
            .max_height(200.0)
            .auto_shrink([false, false]);

        let scroll = if self.auto_scroll {
            scroll_area.stick_to_bottom(true)
        } else {
            scroll_area
        };

        scroll.show(ui, |ui| {
            if filtered_logs.is_empty() {
                ui.colored_label(
                    egui::Color32::GRAY,
                    if logs.is_empty() {
                        "No logs yet."
                    } else {
                        "No logs match the current filter."
                    },
                );
            } else {
                for log in filtered_logs {
                    let color = if log.contains("❌") {
                        egui::Color32::from_rgb(255, 100, 100)
                    } else if log.contains("⚠️") {
                        egui::Color32::from_rgb(255, 200, 100)
                    } else if log.contains("✅") {
                        egui::Color32::from_rgb(100, 255, 100)
                    } else {
                        egui::Color32::LIGHT_GRAY
                    };

                    ui.colored_label(color, log);
                }
            }
        });

        cleared
    }
}

impl Default for ConsolePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for ConsolePanel {
    fn name(&self) -> &str {
        "Console"
    }

    fn show(&mut self, _ui: &mut Ui) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_filter() {
        assert!(LogLevel::All.matches("Any log"));
        assert!(LogLevel::All.matches("⚠️ Warning"));
        assert!(LogLevel::All.matches("❌ Error"));

        assert!(LogLevel::Warning.matches("⚠️ Warning message"));
        assert!(!LogLevel::Warning.matches("Normal message"));
        assert!(!LogLevel::Warning.matches("❌ Error message"));

        assert!(LogLevel::Error.matches("❌ Error message"));
        assert!(!LogLevel::Error.matches("Normal message"));
        assert!(!LogLevel::Error.matches("⚠️ Warning message"));

        assert!(LogLevel::Info.matches("Normal info message"));
        assert!(!LogLevel::Info.matches("⚠️ Warning"));
        assert!(!LogLevel::Info.matches("❌ Error"));
    }

    #[test]
    fn test_console_panel_creation() {
        let panel = ConsolePanel::new();
        assert_eq!(panel.filter_level, LogLevel::All);
        assert!(panel.search_text.is_empty());
        assert!(panel.auto_scroll);
        assert!(panel.show_timestamps);
        assert!(!panel.is_paused);
    }

    #[test]
    fn test_log_level_display_names() {
        assert_eq!(LogLevel::All.name(), "All");
        assert_eq!(LogLevel::Info.name(), "Info");
        assert_eq!(LogLevel::Warning.name(), "Warnings");
        assert_eq!(LogLevel::Error.name(), "Errors");
        assert_eq!(LogLevel::Debug.name(), "Debug");
    }

    #[test]
    fn test_log_level_icons() {
        assert_eq!(LogLevel::All.icon(), "📋");
        assert_eq!(LogLevel::Debug.icon(), "🔍");
        assert_eq!(LogLevel::Info.icon(), "ℹ️");
        assert_eq!(LogLevel::Warning.icon(), "⚠️");
        assert_eq!(LogLevel::Error.icon(), "❌");
    }

    #[test]
    fn test_log_counting_legacy() {
        let logs = vec![
            "Info message".to_string(),
            "⚠️ Warning message".to_string(),
            "❌ Error message".to_string(),
            "Another info".to_string(),
            "❌ Another error".to_string(),
        ];

        let (total, infos, warnings, errors) = ConsolePanel::get_counts_legacy(&logs);
        assert_eq!(total, 5);
        assert_eq!(infos, 2);
        assert_eq!(warnings, 1);
        assert_eq!(errors, 2);
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new("Test message", LogLevel::Info);
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.level, LogLevel::Info);
        assert!(entry.category.is_none());
        assert!(entry.stacktrace.is_none());
    }

    #[test]
    fn test_log_entry_with_category() {
        let entry = LogEntry::new("Test", LogLevel::Warning)
            .with_category("AI")
            .with_source("main.rs", 42);
        
        assert_eq!(entry.category, Some("AI".to_string()));
        assert_eq!(entry.source_file, Some("main.rs".to_string()));
        assert_eq!(entry.source_line, Some(42));
    }

    #[test]
    fn test_push_entry() {
        let mut panel = ConsolePanel::new();
        panel.push_entry(LogEntry::new("Test 1", LogLevel::Info));
        panel.push_entry(LogEntry::new("Test 2", LogLevel::Warning));
        panel.push_entry(LogEntry::new("Test 3", LogLevel::Error));
        
        let (total, debug, info, warnings, errors) = panel.get_counts();
        assert_eq!(total, 3);
        assert_eq!(debug, 0);
        assert_eq!(info, 1);
        assert_eq!(warnings, 1);
        assert_eq!(errors, 1);
    }

    #[test]
    fn test_pause_logging() {
        let mut panel = ConsolePanel::new();
        panel.push_entry(LogEntry::new("Before pause", LogLevel::Info));
        
        panel.is_paused = true;
        panel.push_entry(LogEntry::new("During pause", LogLevel::Info));
        
        assert_eq!(panel.log_entries.len(), 1);
        assert_eq!(panel.paused_entry_count, 1);
        
        panel.is_paused = false;
        panel.push_entry(LogEntry::new("After resume", LogLevel::Info));
        
        assert_eq!(panel.log_entries.len(), 2);
    }

    #[test]
    fn test_max_entries_limit() {
        let mut panel = ConsolePanel::new();
        panel.max_entries = 10;
        
        for i in 0..20 {
            panel.push_entry(LogEntry::new(format!("Log {}", i), LogLevel::Info));
        }
        
        assert_eq!(panel.log_entries.len(), 10);
        // Should have logs 10-19 (first 10 were evicted)
        assert_eq!(panel.log_entries.front().unwrap().message, "Log 10");
        assert_eq!(panel.log_entries.back().unwrap().message, "Log 19");
    }

    #[test]
    fn test_clear_logs() {
        let mut panel = ConsolePanel::new();
        panel.push_entry(LogEntry::new("Test", LogLevel::Info));
        panel.push_entry(LogEntry::new("Test 2", LogLevel::Warning));
        panel.expanded_stacktraces.insert(0);
        
        panel.clear();
        
        assert!(panel.log_entries.is_empty());
        assert!(panel.expanded_stacktraces.is_empty());
    }

    #[test]
    fn test_export_logs() {
        let mut panel = ConsolePanel::new();
        panel.push_entry(LogEntry::new("First log", LogLevel::Info).with_category("Test"));
        panel.push_entry(LogEntry::new("Second log", LogLevel::Warning));
        
        let export = panel.export_logs();
        assert!(export.contains("First log"));
        assert!(export.contains("Second log"));
        assert!(export.contains("[Test]"));
        assert!(export.contains("[Info]"));
        assert!(export.contains("[Warnings]"));
    }

    #[test]
    fn test_filtering_level() {
        let logs = vec![
            "Info".to_string(),
            "⚠️ Warning".to_string(),
            "❌ Error".to_string(),
        ];

        let mut panel = ConsolePanel::new();

        // All
        panel.filter_level = LogLevel::All;
        assert_eq!(panel.filter_logs(&logs).len(), 3);

        // Info
        panel.filter_level = LogLevel::Info;
        let filtered = panel.filter_logs(&logs);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "Info");

        // Warning
        panel.filter_level = LogLevel::Warning;
        let filtered = panel.filter_logs(&logs);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "⚠️ Warning");

        // Error
        panel.filter_level = LogLevel::Error;
        let filtered = panel.filter_logs(&logs);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "❌ Error");
    }

    #[test]
    fn test_filtering_search() {
        let logs = vec![
            "Apple".to_string(),
            "Banana".to_string(),
            "Cherry".to_string(),
        ];

        let mut panel = ConsolePanel::new();
        panel.filter_level = LogLevel::All;

        // Case insensitive match
        panel.search_text = "an".to_string();
        let filtered = panel.filter_logs(&logs);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "Banana");

        // No match
        panel.search_text = "xyz".to_string();
        assert!(panel.filter_logs(&logs).is_empty());

        // Empty search
        panel.search_text = "".to_string();
        assert_eq!(panel.filter_logs(&logs).len(), 3);
    }

    #[test]
    fn test_filtering_combined() {
        let logs = vec![
            "Info Apple".to_string(),
            "⚠️ Warning Apple".to_string(),
            "❌ Error Banana".to_string(),
        ];

        let mut panel = ConsolePanel::new();
        
        // Filter Info + Search "Apple"
        panel.filter_level = LogLevel::Info;
        panel.search_text = "Apple".to_string();
        let filtered = panel.filter_logs(&logs);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "Info Apple");

        // Filter Warning + Search "Apple"
        panel.filter_level = LogLevel::Warning;
        let filtered = panel.filter_logs(&logs);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "⚠️ Warning Apple");

        // Filter Error + Search "Apple" (Mismatch)
        panel.filter_level = LogLevel::Error;
        assert!(panel.filter_logs(&logs).is_empty());
    }

    #[test]
    fn test_command_history() {
        let mut panel = ConsolePanel::new();
        panel.execute_command("help");
        panel.execute_command("clear");
        
        assert_eq!(panel.command_history.len(), 2);
        assert_eq!(panel.command_history[0], "clear");
        assert_eq!(panel.command_history[1], "help");
    }

    #[test]
    fn test_filter_command() {
        let mut panel = ConsolePanel::new();
        
        panel.execute_command("filter warning");
        assert_eq!(panel.filter_level, LogLevel::Warning);
        
        panel.execute_command("filter error");
        assert_eq!(panel.filter_level, LogLevel::Error);
        
        panel.execute_command("filter all");
        assert_eq!(panel.filter_level, LogLevel::All);
    }

    #[test]
    fn test_category_tracking() {
        let mut panel = ConsolePanel::new();
        
        panel.push_entry(LogEntry::new("Test", LogLevel::Info).with_category("AI"));
        panel.push_entry(LogEntry::new("Test", LogLevel::Info).with_category("Physics"));
        panel.push_entry(LogEntry::new("Test", LogLevel::Info).with_category("AI"));
        
        assert_eq!(panel.all_categories.len(), 2);
        assert!(panel.all_categories.contains(&"AI".to_string()));
        assert!(panel.all_categories.contains(&"Physics".to_string()));
    }

    // ========== LogLevel Display + Hash Tests ==========

    #[test]
    fn test_log_level_display() {
        for level in LogLevel::all_levels() {
            let display = format!("{}", level);
            assert!(display.contains(level.name()));
        }
    }

    #[test]
    fn test_log_level_all_levels_variants() {
        let all = LogLevel::all_levels();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&LogLevel::All));
        assert!(all.contains(&LogLevel::Debug));
        assert!(all.contains(&LogLevel::Info));
        assert!(all.contains(&LogLevel::Warning));
        assert!(all.contains(&LogLevel::Error));
    }

    #[test]
    fn test_log_level_hash() {
        use std::collections::HashSet;
        let set: HashSet<LogLevel> = LogLevel::all_levels().iter().copied().collect();
        assert_eq!(set.len(), 5);
    }

    #[test]
    fn test_log_level_default() {
        assert_eq!(LogLevel::default(), LogLevel::Info);
    }
}
