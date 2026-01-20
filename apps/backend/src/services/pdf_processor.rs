//! PDF Processing Service
//!
//! Extracts text content from PDF files for flashcard generation.
//! Uses pdf-extract crate for text extraction.

use thiserror::Error;

/// Errors that can occur during PDF processing
#[derive(Error, Debug)]
pub enum PdfError {
    #[error("Failed to extract text from PDF: {0}")]
    ExtractionError(String),

    #[error("PDF is empty or contains no extractable text")]
    EmptyContent,

    #[error("PDF is too large (max {max_mb}MB)")]
    TooLarge { max_mb: usize },

    #[error("Invalid PDF format: {0}")]
    InvalidFormat(String),

    #[error("Base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),
}

/// Configuration for PDF processing
#[derive(Debug, Clone)]
pub struct PdfConfig {
    /// Maximum PDF file size in bytes
    pub max_size: usize,
    /// Minimum text length to consider valid
    pub min_text_length: usize,
    /// Maximum text length to return (truncate if larger)
    pub max_text_length: usize,
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            max_size: 50 * 1024 * 1024, // 50MB
            min_text_length: 100,
            max_text_length: 500_000, // ~500KB of text
        }
    }
}

/// Result of PDF text extraction
#[derive(Debug)]
pub struct PdfContent {
    /// Extracted text content
    pub text: String,
    /// Number of pages in the PDF
    pub page_count: usize,
    /// Whether the text was truncated
    pub truncated: bool,
    /// Extracted title if available
    pub title: Option<String>,
}

/// Extract text content from a PDF file
///
/// # Arguments
/// * `pdf_bytes` - Raw PDF file bytes
/// * `config` - Optional configuration (uses defaults if None)
///
/// # Returns
/// * `Ok(PdfContent)` - Extracted text and metadata
/// * `Err(PdfError)` - If extraction fails
pub fn extract_text(pdf_bytes: &[u8], config: Option<&PdfConfig>) -> Result<PdfContent, PdfError> {
    let config = config.cloned().unwrap_or_default();

    // Check file size
    if pdf_bytes.len() > config.max_size {
        return Err(PdfError::TooLarge {
            max_mb: config.max_size / (1024 * 1024),
        });
    }

    // Extract text using pdf-extract
    let text = pdf_extract::extract_text_from_mem(pdf_bytes)
        .map_err(|e| PdfError::ExtractionError(e.to_string()))?;

    // Clean up the extracted text
    let cleaned_text = clean_text(&text);

    if cleaned_text.len() < config.min_text_length {
        return Err(PdfError::EmptyContent);
    }

    // Truncate if necessary
    let (final_text, truncated) = if cleaned_text.len() > config.max_text_length {
        // Try to truncate at a sentence boundary
        let truncated_text = truncate_at_sentence(&cleaned_text, config.max_text_length);
        (truncated_text, true)
    } else {
        (cleaned_text, false)
    };

    // Try to extract page count (rough estimate from text)
    let page_count = estimate_page_count(&text);

    // Try to extract title from first line
    let title = extract_title(&final_text);

    Ok(PdfContent {
        text: final_text,
        page_count,
        truncated,
        title,
    })
}

/// Extract text from base64-encoded PDF data
pub fn extract_text_from_base64(
    base64_data: &str,
    config: Option<&PdfConfig>,
) -> Result<PdfContent, PdfError> {
    use base64::Engine;

    let pdf_bytes = base64::engine::general_purpose::STANDARD.decode(base64_data)?;
    extract_text(&pdf_bytes, config)
}

/// Clean up extracted text by normalizing whitespace and removing artifacts
fn clean_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut prev_was_whitespace = false;
    let mut prev_was_newline = false;

    for c in text.chars() {
        match c {
            // Normalize various whitespace to single space
            ' ' | '\t' => {
                if !prev_was_whitespace {
                    result.push(' ');
                    prev_was_whitespace = true;
                }
                prev_was_newline = false;
            }
            // Keep paragraph breaks (double newlines)
            '\n' | '\r' => {
                if prev_was_newline {
                    if !result.ends_with("\n\n") {
                        result.push_str("\n\n");
                    }
                } else if !prev_was_whitespace {
                    result.push(' ');
                }
                prev_was_whitespace = true;
                prev_was_newline = true;
            }
            // Filter out control characters but keep printable ones
            c if c.is_control() => {
                continue;
            }
            // Keep regular characters
            _ => {
                result.push(c);
                prev_was_whitespace = false;
                prev_was_newline = false;
            }
        }
    }

    // Remove any leading/trailing whitespace
    result.trim().to_string()
}

/// Truncate text at a sentence boundary
fn truncate_at_sentence(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }

    // Look for sentence end within the last 10% of the allowed length
    let search_start = (max_len as f64 * 0.9) as usize;
    let search_region = &text[search_start..max_len];

    // Find the last sentence-ending punctuation
    let sentence_end = search_region
        .rfind(|c| c == '.' || c == '!' || c == '?')
        .map(|pos| search_start + pos + 1);

    match sentence_end {
        Some(pos) => text[..pos].trim().to_string() + "...",
        None => {
            // No sentence boundary found, truncate at word boundary
            let word_end = text[..max_len]
                .rfind(char::is_whitespace)
                .unwrap_or(max_len);
            text[..word_end].trim().to_string() + "..."
        }
    }
}

/// Estimate page count from text (rough heuristic)
fn estimate_page_count(text: &str) -> usize {
    // Assume approximately 2000-3000 characters per page
    let chars_per_page = 2500;
    (text.len() / chars_per_page).max(1)
}

/// Try to extract a title from the beginning of the text
fn extract_title(text: &str) -> Option<String> {
    // Take the first line that looks like a title
    // (short line followed by longer content)
    let lines: Vec<&str> = text.lines().take(5).collect();

    if lines.is_empty() {
        return None;
    }

    let first_line = lines[0].trim();

    // Heuristics for title detection:
    // - Short enough (less than 100 chars)
    // - Doesn't end with common sentence punctuation
    // - Has some content
    if first_line.len() >= 5
        && first_line.len() <= 100
        && !first_line.ends_with('.')
        && !first_line.ends_with(',')
    {
        Some(first_line.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text() {
        let input = "Hello   world\n\n\nThis is   a test.";
        let cleaned = clean_text(input);
        assert!(!cleaned.contains("   ")); // No triple spaces
        assert!(cleaned.contains("Hello world")); // Normalized spaces
    }

    #[test]
    fn test_truncate_at_sentence() {
        let text = "First sentence. Second sentence. Third sentence.";
        let truncated = truncate_at_sentence(text, 25);
        assert!(truncated.ends_with("..."));
        assert!(truncated.len() <= 30); // Allow some room for "..."
    }

    #[test]
    fn test_extract_title() {
        let text = "Introduction to Machine Learning\n\nThis document discusses...";
        let title = extract_title(text);
        assert_eq!(title, Some("Introduction to Machine Learning".to_string()));
    }

    #[test]
    fn test_extract_title_no_title() {
        let text = "This is a very long first line that goes on and on and probably isn't a title because it's way too long to be a reasonable document title.";
        let title = extract_title(text);
        assert!(title.is_none());
    }

    #[test]
    fn test_estimate_page_count() {
        let short_text = "Short text";
        assert_eq!(estimate_page_count(short_text), 1);

        let long_text = "x".repeat(10000);
        assert_eq!(estimate_page_count(&long_text), 4);
    }
}
