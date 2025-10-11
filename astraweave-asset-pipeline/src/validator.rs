//! Asset validation for quality checks and CI integration

use anyhow::{Context, Result};
use std::path::Path;

/// Asset validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Asset path
    pub path: String,
    /// Validation passed
    pub passed: bool,
    /// Warnings (non-critical issues)
    pub warnings: Vec<String>,
    /// Errors (critical issues)
    pub errors: Vec<String>,
    /// Metrics (size, quality, etc.)
    pub metrics: ValidationMetrics,
}

/// Validation metrics
#[derive(Debug, Clone, Default)]
pub struct ValidationMetrics {
    /// Original file size (bytes)
    pub original_size: usize,
    /// Processed file size (bytes)
    pub processed_size: usize,
    /// Compression ratio (if applicable)
    pub compression_ratio: Option<f32>,
    /// Quality score (0.0-1.0, higher is better)
    pub quality_score: Option<f32>,
}

impl ValidationReport {
    /// Create a new validation report
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            passed: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            metrics: ValidationMetrics::default(),
        }
    }
    
    /// Add a warning
    pub fn add_warning(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }
    
    /// Add an error (marks validation as failed)
    pub fn add_error(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
        self.passed = false;
    }
    
    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.passed
    }
    
    /// Get summary string
    pub fn summary(&self) -> String {
        if self.passed {
            format!("✅ {} - PASSED ({} warnings)", self.path, self.warnings.len())
        } else {
            format!("❌ {} - FAILED ({} errors, {} warnings)", 
                self.path, self.errors.len(), self.warnings.len())
        }
    }
}

/// Asset validator
pub struct AssetValidator {
    /// Maximum allowed size (bytes)
    pub max_size: Option<usize>,
    /// Minimum quality threshold (0.0-1.0)
    pub min_quality: Option<f32>,
    /// Require compression
    pub require_compression: bool,
    /// Minimum compression ratio (if compression required)
    pub min_compression_ratio: Option<f32>,
}

impl Default for AssetValidator {
    fn default() -> Self {
        Self {
            max_size: Some(50 * 1024 * 1024), // 50 MB default
            min_quality: Some(0.90), // 90% quality minimum
            require_compression: false,
            min_compression_ratio: Some(2.0), // 2:1 compression minimum
        }
    }
}

impl AssetValidator {
    /// Create a new validator with default settings
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Validate a texture asset
    pub fn validate_texture(&self, path: &Path, original_size: usize, compressed_size: usize) -> ValidationReport {
        let mut report = ValidationReport::new(path.display().to_string());
        
        report.metrics.original_size = original_size;
        report.metrics.processed_size = compressed_size;
        
        // Check size limits
        if let Some(max_size) = self.max_size {
            if compressed_size > max_size {
                report.add_error(format!(
                    "Compressed size ({} bytes) exceeds limit ({} bytes)",
                    compressed_size, max_size
                ));
            }
        }
        
        // Check compression ratio
        let compression_ratio = original_size as f32 / compressed_size.max(1) as f32;
        report.metrics.compression_ratio = Some(compression_ratio);
        
        if self.require_compression {
            if let Some(min_ratio) = self.min_compression_ratio {
                if compression_ratio < min_ratio {
                    report.add_error(format!(
                        "Compression ratio ({:.2}:1) below minimum ({:.2}:1)",
                        compression_ratio, min_ratio
                    ));
                }
            }
        }
        
        // Estimate quality (simplified: higher compression = lower quality)
        // BC7 typically achieves 4:1 with minimal quality loss
        let quality_score = if compression_ratio <= 4.0 {
            1.0 // Excellent quality
        } else if compression_ratio <= 8.0 {
            0.95 // Good quality
        } else {
            0.80 // Acceptable quality
        };
        
        report.metrics.quality_score = Some(quality_score);
        
        if let Some(min_quality) = self.min_quality {
            if quality_score < min_quality {
                report.add_warning(format!(
                    "Quality score ({:.2}) below threshold ({:.2})",
                    quality_score, min_quality
                ));
            }
        }
        
        report
    }
    
    /// Validate a mesh asset
    pub fn validate_mesh(&self, path: &Path, vertex_count: usize, acmr: f32) -> ValidationReport {
        let mut report = ValidationReport::new(path.display().to_string());
        
        report.metrics.original_size = vertex_count;
        
        // Check ACMR (Average Cache Miss Ratio)
        // Lower is better: < 0.5 excellent, < 1.0 good, < 1.5 acceptable
        if acmr > 1.5 {
            report.add_warning(format!(
                "High ACMR ({:.2}), mesh may have poor cache utilization",
                acmr
            ));
        }
        
        if acmr < 0.5 {
            // Excellent cache utilization
        } else if acmr < 1.0 {
            // Good
        } else if acmr < 1.5 {
            // Acceptable
        } else {
            // Poor - already warned above
        }
        
        // Estimate quality based on ACMR
        let quality_score = (1.5 - acmr.min(1.5)) / 1.5;
        report.metrics.quality_score = Some(quality_score.max(0.0));
        
        report
    }
    
    /// Validate multiple assets and generate summary report
    pub fn validate_batch(&self, reports: &[ValidationReport]) -> BatchValidationReport {
        let total = reports.len();
        let passed = reports.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let total_warnings = reports.iter().map(|r| r.warnings.len()).sum();
        let total_errors = reports.iter().map(|r| r.errors.len()).sum();
        
        BatchValidationReport {
            total_assets: total,
            passed_assets: passed,
            failed_assets: failed,
            total_warnings,
            total_errors,
            reports: reports.to_vec(),
        }
    }
}

/// Batch validation report (for CI)
#[derive(Debug, Clone)]
pub struct BatchValidationReport {
    /// Total assets validated
    pub total_assets: usize,
    /// Assets that passed
    pub passed_assets: usize,
    /// Assets that failed
    pub failed_assets: usize,
    /// Total warnings
    pub total_warnings: usize,
    /// Total errors
    pub total_errors: usize,
    /// Individual reports
    pub reports: Vec<ValidationReport>,
}

impl BatchValidationReport {
    /// Check if all assets passed
    pub fn all_passed(&self) -> bool {
        self.failed_assets == 0
    }
    
    /// Get summary string
    pub fn summary(&self) -> String {
        if self.all_passed() {
            format!(
                "✅ All {} assets passed ({} warnings)",
                self.total_assets, self.total_warnings
            )
        } else {
            format!(
                "❌ {} of {} assets failed ({} errors, {} warnings)",
                self.failed_assets, self.total_assets,
                self.total_errors, self.total_warnings
            )
        }
    }
    
    /// Print detailed report
    pub fn print(&self) {
        println!("\n{}", "=".repeat(60));
        println!("Asset Validation Report");
        println!("{}", "=".repeat(60));
        println!("Total assets: {}", self.total_assets);
        println!("Passed: {} ✅", self.passed_assets);
        println!("Failed: {} ❌", self.failed_assets);
        println!("Warnings: {} ⚠️", self.total_warnings);
        println!("Errors: {} 🔴", self.total_errors);
        println!("{}", "=".repeat(60));
        
        // Print individual reports
        for report in &self.reports {
            println!("\n{}", report.summary());
            
            for warning in &report.warnings {
                println!("  ⚠️  {}", warning);
            }
            
            for error in &report.errors {
                println!("  🔴 {}", error);
            }
            
            // Print metrics
            let m = &report.metrics;
            if let Some(ratio) = m.compression_ratio {
                println!("  📊 Compression: {:.2}:1 ({} → {} bytes)",
                    ratio, m.original_size, m.processed_size);
            }
            if let Some(quality) = m.quality_score {
                println!("  📊 Quality: {:.1}%", quality * 100.0);
            }
        }
        
        println!("\n{}", "=".repeat(60));
        println!("{}", self.summary());
        println!("{}", "=".repeat(60));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new("test.png");
        assert!(report.is_valid());
        
        report.add_warning("Minor issue");
        assert!(report.is_valid()); // Warnings don't fail validation
        assert_eq!(report.warnings.len(), 1);
        
        report.add_error("Critical issue");
        assert!(!report.is_valid()); // Errors fail validation
        assert_eq!(report.errors.len(), 1);
    }

    #[test]
    fn test_texture_validation_pass() {
        let validator = AssetValidator::new();
        let path = PathBuf::from("test.png");
        
        // Good compression: 4:1 ratio (1024 → 256 bytes)
        let report = validator.validate_texture(&path, 1024, 256);
        assert!(report.is_valid());
        assert_eq!(report.errors.len(), 0);
    }

    #[test]
    fn test_texture_validation_size_limit() {
        let mut validator = AssetValidator::new();
        validator.max_size = Some(100); // 100 bytes limit
        
        let path = PathBuf::from("test.png");
        
        // Compressed size exceeds limit
        let report = validator.validate_texture(&path, 1024, 200);
        assert!(!report.is_valid());
        assert!(report.errors.iter().any(|e| e.contains("exceeds limit")));
    }

    #[test]
    fn test_texture_validation_compression_ratio() {
        let mut validator = AssetValidator::new();
        validator.require_compression = true;
        validator.min_compression_ratio = Some(4.0); // Require 4:1 compression
        
        let path = PathBuf::from("test.png");
        
        // Only 2:1 compression (below threshold)
        let report = validator.validate_texture(&path, 1024, 512);
        assert!(!report.is_valid());
        assert!(report.errors.iter().any(|e| e.contains("below minimum")));
    }

    #[test]
    fn test_mesh_validation() {
        let validator = AssetValidator::new();
        let path = PathBuf::from("test.obj");
        
        // Good ACMR (0.5)
        let report = validator.validate_mesh(&path, 1000, 0.5);
        assert!(report.is_valid());
        assert_eq!(report.warnings.len(), 0);
        
        // Poor ACMR (2.0)
        let report = validator.validate_mesh(&path, 1000, 2.0);
        assert!(report.is_valid()); // Still valid, but has warning
        assert!(report.warnings.iter().any(|w| w.contains("High ACMR")));
    }

    #[test]
    fn test_batch_validation() {
        let validator = AssetValidator::new();
        
        let mut reports = vec![
            ValidationReport::new("asset1.png"),
            ValidationReport::new("asset2.png"),
        ];
        
        reports[0].add_warning("Minor issue");
        reports[1].add_error("Critical issue");
        
        let batch = validator.validate_batch(&reports);
        
        assert_eq!(batch.total_assets, 2);
        assert_eq!(batch.passed_assets, 1);
        assert_eq!(batch.failed_assets, 1);
        assert_eq!(batch.total_warnings, 1);
        assert_eq!(batch.total_errors, 1);
        assert!(!batch.all_passed());
    }
}
