use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::McpTool;

/// A row of CSV data represented as key-value pairs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Row {
    pub fields: Vec<(String, String)>,
}

/// Descriptive statistics for a numeric dataset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Stats {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
}

/// The result of a chart generation request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChartResult {
    pub chart_type: String,
    pub data_points: usize,
    /// Base64-encoded image data (stub returns an empty string).
    pub image_data: String,
}

/// MCP tool server interface for data processing and analysis.
///
/// Stub implementation — returns mock data structures.
pub struct DataTool;

impl DataTool {
    pub fn new() -> Self {
        Self
    }

    /// Parse a CSV file and return its rows.
    ///
    /// Stub — returns a single mock row regardless of path.
    pub fn parse_csv(&self, _path: &str) -> Vec<Row> {
        vec![Row {
            fields: vec![
                ("id".to_string(), "1".to_string()),
                ("name".to_string(), "mock".to_string()),
                ("value".to_string(), "42".to_string()),
            ],
        }]
    }

    /// Parse a JSON file and return its contents as a serde_json `Value`.
    ///
    /// Stub — returns a simple mock JSON object.
    pub fn parse_json(&self, _path: &str) -> Value {
        serde_json::json!({
            "mock": true,
            "source": "data-tool-stub"
        })
    }

    /// Compute descriptive statistics for a list of numeric values.
    ///
    /// Stub — returns mock statistics.
    pub fn compute_stats(&self, data: &[f64]) -> Stats {
        if data.is_empty() {
            return Stats {
                count: 0,
                mean: 0.0,
                median: 0.0,
                min: 0.0,
                max: 0.0,
                std_dev: 0.0,
            };
        }

        let count = data.len();
        let sum: f64 = data.iter().sum();
        let mean = sum / count as f64;

        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = if count % 2 == 0 {
            (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0
        } else {
            sorted[count / 2]
        };

        let min = sorted[0];
        let max = sorted[count - 1];

        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        Stats {
            count,
            mean,
            median,
            min,
            max,
            std_dev,
        }
    }

    /// Generate a chart from data.
    ///
    /// Stub — returns a `ChartResult` with an empty image.
    pub fn generate_chart(&self, data: &[f64], chart_type: &str) -> ChartResult {
        ChartResult {
            chart_type: chart_type.to_string(),
            data_points: data.len(),
            image_data: String::new(),
        }
    }
}

impl Default for DataTool {
    fn default() -> Self {
        Self::new()
    }
}

impl McpTool for DataTool {
    fn name(&self) -> &str {
        "data"
    }

    fn risk_level(&self) -> &str {
        "low"
    }

    fn description(&self) -> &str {
        "Parse CSV/JSON data, compute statistics, and generate charts"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_returns_mock_rows() {
        let tool = DataTool::new();
        let rows = tool.parse_csv("/tmp/data.csv");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].fields.len(), 3);
        assert_eq!(rows[0].fields[0].0, "id");
    }

    #[test]
    fn test_compute_stats_correct() {
        let tool = DataTool::new();
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let stats = tool.compute_stats(&data);

        assert_eq!(stats.count, 8);
        assert!((stats.mean - 5.0).abs() < 0.001);
        assert!((stats.median - 4.5).abs() < 0.001);
        assert!((stats.min - 2.0).abs() < f64::EPSILON);
        assert!((stats.max - 9.0).abs() < f64::EPSILON);
        assert!((stats.std_dev - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_stats_empty() {
        let tool = DataTool::new();
        let stats = tool.compute_stats(&[]);
        assert_eq!(stats.count, 0);
        assert!((stats.mean - 0.0).abs() < f64::EPSILON);
    }
}
