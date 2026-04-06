use scraper::{ElementRef, Html, Selector};
use serde_json::Value;

/// Safety limits for colspan/rowspan to prevent DoS
const MAX_COLSPAN: usize = 100;
const MAX_ROWSPAN: usize = 1000;

/// Extract tables from HTML content using a 2D grid algorithm
/// that properly handles colspan and rowspan attributes.
pub fn extract(html: &str) -> Vec<Value> {
    let document = Html::parse_document(html);
    let mut tables = Vec::new();

    let table_selector = match Selector::parse("table") {
        Ok(s) => s,
        Err(_) => return tables,
    };

    for table in document.select(&table_selector) {
        let table_data = extract_single_table(&table);
        if let Some(data) = table_data {
            tables.push(data);
        }
    }

    tables
}

/// Extract a single table element into structured JSON
fn extract_single_table(table: &ElementRef) -> Option<Value> {
    let mut table_data = serde_json::json!({
        "headers": [],
        "rows": [],
        "caption": null
    });

    // Extract caption if present (direct child only)
    if let Ok(caption_selector) = Selector::parse("caption") {
        for caption in table.select(&caption_selector) {
            // Only consider direct child captions (not from nested tables)
            if is_direct_child_of_table(&caption, table) {
                table_data["caption"] = Value::String(
                    caption
                        .text()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .trim()
                        .to_string(),
                );
                break;
            }
        }
    }

    // Collect all direct-child rows (not from nested tables)
    let all_rows = collect_direct_rows(table);
    if all_rows.is_empty() {
        return None;
    }

    // Determine which rows are header rows
    let (header_row_count, has_thead) = detect_header_rows(table, &all_rows);

    // Build the 2D grid
    let grid = build_grid(&all_rows);
    if grid.is_empty() {
        return None;
    }

    // Split into headers and data rows
    let header_rows = if header_row_count > 0 && header_row_count <= grid.len() {
        header_row_count
    } else if !has_thead {
        // If first row is all <th> cells, treat as header
        if is_all_th(&all_rows[0]) { 1 } else { 0 }
    } else {
        0
    };

    if header_rows > 0 {
        // Use the first header row as headers (flatten multiple header rows)
        let headers: Vec<Value> = grid[0]
            .iter()
            .map(|cell| Value::String(cell.clone().unwrap_or_default()))
            .collect();
        table_data["headers"] = Value::Array(headers);
    }

    // Data rows
    let data_start = if header_rows > 0 { header_rows } else { 0 };
    let mut rows = Vec::new();
    for row in grid.iter().skip(data_start) {
        let cells: Vec<Value> = row
            .iter()
            .map(|cell| Value::String(cell.clone().unwrap_or_default()))
            .collect();

        // Skip entirely empty rows
        if !cells
            .iter()
            .all(|c| c.as_str().is_none_or(|s| s.is_empty()))
        {
            rows.push(Value::Array(cells));
        }
    }

    table_data["rows"] = Value::Array(rows);

    // Only include tables with meaningful content
    if table_data["rows"].as_array().is_none_or(|r| r.is_empty())
        && table_data["headers"]
            .as_array()
            .is_none_or(|h| h.is_empty())
    {
        return None;
    }

    Some(table_data)
}

/// Check if an element is a direct child of the given table
/// (not inside a nested table)
fn is_direct_child_of_table(element: &ElementRef, table: &ElementRef) -> bool {
    let table_id = table.id();
    let mut current = element.parent();
    while let Some(node) = current {
        if let Some(el) = ElementRef::wrap(node)
            && el.value().name() == "table"
        {
            return el.id() == table_id;
        }
        current = node.parent();
    }
    false
}

/// Collect all direct <tr> elements from a table, respecting
/// thead/tbody/tfoot structure but excluding nested table rows
fn collect_direct_rows<'a>(table: &'a ElementRef<'a>) -> Vec<ElementRef<'a>> {
    let mut rows = Vec::new();

    let tr_selector = match Selector::parse("tr") {
        Ok(s) => s,
        Err(_) => return rows,
    };

    for tr in table.select(&tr_selector) {
        // Only include rows that are direct children of this table
        // (not rows from nested tables)
        if is_direct_child_of_table(&tr, table) {
            rows.push(tr);
        }
    }

    rows
}

/// Detect how many header rows exist
fn detect_header_rows(table: &ElementRef, rows: &[ElementRef]) -> (usize, bool) {
    let thead_selector = match Selector::parse("thead") {
        Ok(s) => s,
        Err(_) => return (0, false),
    };

    // Check for <thead>
    let has_thead = table.select(&thead_selector).next().is_some();

    if has_thead {
        // Count rows inside thead
        let tr_selector = match Selector::parse("thead > tr") {
            Ok(s) => s,
            Err(_) => return (0, true),
        };
        let thead_row_count = table
            .select(&tr_selector)
            .filter(|tr| is_direct_child_of_table(tr, table))
            .count();
        return (thead_row_count, true);
    }

    // No thead - check if first row is all <th>
    if !rows.is_empty() && is_all_th(&rows[0]) {
        return (1, false);
    }

    (0, false)
}

/// Check if all cells in a row are <th> elements
fn is_all_th(row: &ElementRef) -> bool {
    let cell_selector = match Selector::parse("td, th") {
        Ok(s) => s,
        Err(_) => return false,
    };

    let mut has_cells = false;
    for cell in row.select(&cell_selector) {
        has_cells = true;
        if cell.value().name() != "th" {
            return false;
        }
    }
    has_cells
}

/// Build a 2D grid from table rows, handling colspan and rowspan
fn build_grid(rows: &[ElementRef]) -> Vec<Vec<Option<String>>> {
    if rows.is_empty() {
        return Vec::new();
    }

    // First pass: determine the max number of columns
    let mut max_cols = 0;
    for row in rows {
        let mut col_count = 0;
        let cell_selector = match Selector::parse("td, th") {
            Ok(s) => s,
            Err(_) => continue,
        };
        for cell in row.select(&cell_selector) {
            let colspan = parse_span_attr(&cell, "colspan").min(MAX_COLSPAN);
            col_count += colspan;
        }
        if col_count > max_cols {
            max_cols = col_count;
        }
    }

    if max_cols == 0 {
        return Vec::new();
    }

    // Safety limit on columns
    max_cols = max_cols.min(MAX_COLSPAN);

    let num_rows = rows.len();
    // Initialize grid with None (empty cells)
    let mut grid: Vec<Vec<Option<String>>> = vec![vec![None; max_cols]; num_rows];

    // Second pass: fill the grid with cell contents
    for (row_idx, row) in rows.iter().enumerate() {
        let cell_selector = match Selector::parse("td, th") {
            Ok(s) => s,
            Err(_) => continue,
        };

        let mut col_idx = 0;

        for cell in row.select(&cell_selector) {
            // Skip columns already occupied by prior rowspans
            while col_idx < max_cols && grid[row_idx][col_idx].is_some() {
                col_idx += 1;
            }

            if col_idx >= max_cols {
                break;
            }

            let colspan = parse_span_attr(&cell, "colspan").min(MAX_COLSPAN);
            let rowspan = parse_span_attr(&cell, "rowspan").min(MAX_ROWSPAN);

            let text = cell.text().collect::<Vec<_>>().join(" ").trim().to_string();

            // Fill grid positions for this cell's span
            for dr in 0..rowspan {
                let target_row = row_idx + dr;
                if target_row >= num_rows {
                    break;
                }
                for dc in 0..colspan {
                    let target_col = col_idx + dc;
                    if target_col >= max_cols {
                        break;
                    }
                    if grid[target_row][target_col].is_none() {
                        if dr == 0 && dc == 0 {
                            grid[target_row][target_col] = Some(text.clone());
                        } else {
                            // Spanned cells get empty string (not the content)
                            grid[target_row][target_col] = Some(String::new());
                        }
                    }
                }
            }

            col_idx += colspan;
        }
    }

    // Fill any remaining None cells with empty string
    for row in &mut grid {
        for cell in row.iter_mut() {
            if cell.is_none() {
                *cell = Some(String::new());
            }
        }
    }

    grid
}

/// Parse a colspan or rowspan attribute value, defaulting to 1
fn parse_span_attr(cell: &ElementRef, attr_name: &str) -> usize {
    cell.value()
        .attr(attr_name)
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(1)
        .max(1) // Minimum span is 1
}
