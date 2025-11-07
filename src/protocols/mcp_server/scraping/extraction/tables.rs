use scraper::{Html, Selector};
use serde_json::Value;

/// Extract tables from HTML content
pub fn extract(html: &str) -> Vec<Value> {
    let document = Html::parse_document(html);
    let mut tables = Vec::new();

    // Select all tables
    if let Ok(table_selector) = Selector::parse("table") {
        for table in document.select(&table_selector) {
            let mut table_data = serde_json::json!({
                "headers": [],
                "rows": [],
                "caption": null
            });

            // Extract caption if present
            if let Ok(caption_selector) = Selector::parse("caption") {
                if let Some(caption) = table.select(&caption_selector).next() {
                    table_data["caption"] = Value::String(
                        caption.text().collect::<Vec<_>>().join(" ").trim().to_string()
                    );
                }
            }

            // Extract headers from thead or first tr
            if let Ok(thead_selector) = Selector::parse("thead tr th, thead tr td") {
                let headers: Vec<String> = table.select(&thead_selector)
                    .map(|th| th.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .collect();

                if !headers.is_empty() {
                    table_data["headers"] = Value::Array(
                        headers.into_iter().map(Value::String).collect()
                    );
                }
            }

            // If no headers in thead, try first tr
            if table_data["headers"].as_array().map_or(true, |h| h.is_empty()) {
                if let Ok(first_row_selector) = Selector::parse("tr:first-child th, tr:first-child td") {
                    let headers: Vec<String> = table.select(&first_row_selector)
                        .map(|th| th.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        .collect();

                    if !headers.is_empty() {
                        table_data["headers"] = Value::Array(
                            headers.into_iter().map(Value::String).collect()
                        );
                    }
                }
            }

            // Extract data rows
            if let Ok(row_selector) = Selector::parse("tbody tr, tr") {
                let mut rows = Vec::new();
                let mut skip_first = table_data["headers"].as_array().map_or(false, |h| !h.is_empty());

                for row in table.select(&row_selector) {
                    if skip_first {
                        skip_first = false;
                        continue;
                    }

                    if let Ok(cell_selector) = Selector::parse("td, th") {
                        let cells: Vec<String> = row.select(&cell_selector)
                            .map(|td| td.text().collect::<Vec<_>>().join(" ").trim().to_string())
                            .collect();

                        if !cells.is_empty() && !cells.iter().all(|c| c.is_empty()) {
                            rows.push(Value::Array(
                                cells.into_iter().map(Value::String).collect()
                            ));
                        }
                    }
                }

                table_data["rows"] = Value::Array(rows);
            }

            // Only include tables with meaningful content
            if table_data["rows"].as_array().map_or(false, |rows| !rows.is_empty()) {
                tables.push(table_data);
            }
        }
    }

    tables
}
