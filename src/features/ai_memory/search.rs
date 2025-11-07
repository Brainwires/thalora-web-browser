use super::types::*;

/// Search research entries
pub(super) fn search_research<'a>(
    memory_data: &'a MemoryData,
    criteria: &'a MemorySearchCriteria,
) -> Vec<(&'a String, &'a ResearchEntry)> {
    let mut results: Vec<_> = memory_data
        .research
        .iter()
        .filter(|(_, entry)| matches_research_criteria(entry, criteria))
        .collect();

    sort_research_results(&mut results, &criteria.sort_by);

    if let Some(limit) = criteria.limit {
        results.truncate(limit);
    }

    results
}

fn matches_research_criteria(entry: &ResearchEntry, criteria: &MemorySearchCriteria) -> bool {
    // Query matching
    if let Some(query) = &criteria.query {
        let query_lower = query.to_lowercase();
        if !entry.topic.to_lowercase().contains(&query_lower)
            && !entry.summary.to_lowercase().contains(&query_lower)
            && !entry
                .findings
                .iter()
                .any(|f| f.to_lowercase().contains(&query_lower))
        {
            return false;
        }
    }

    // Tag matching
    if let Some(tags) = &criteria.tags {
        if !tags.iter().any(|tag| entry.tags.contains(tag)) {
            return false;
        }
    }

    // Date range matching
    if let Some((start, end)) = &criteria.date_range {
        if entry.created_at < *start || entry.created_at > *end {
            return false;
        }
    }

    true
}

fn sort_research_results(results: &mut Vec<(&String, &ResearchEntry)>, sort_by: &MemorySortBy) {
    match sort_by {
        MemorySortBy::CreatedAt => results.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at)),
        MemorySortBy::UpdatedAt => results.sort_by(|a, b| b.1.updated_at.cmp(&a.1.updated_at)),
        MemorySortBy::Relevance => results.sort_by(|a, b| {
            b.1.confidence_score
                .partial_cmp(&a.1.confidence_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        _ => {} // Other sort types handled in specific contexts
    }
}

/// Search bookmarks
pub(super) fn search_bookmarks<'a>(
    memory_data: &'a MemoryData,
    criteria: &'a MemorySearchCriteria,
) -> Vec<(&'a String, &'a BookmarkEntry)> {
    let mut results: Vec<_> = memory_data
        .bookmarks
        .iter()
        .filter(|(_, entry)| matches_bookmark_criteria(entry, criteria))
        .collect();

    // Sort by access count for bookmarks
    results.sort_by(|a, b| b.1.access_count.cmp(&a.1.access_count));

    if let Some(limit) = criteria.limit {
        results.truncate(limit);
    }

    results
}

fn matches_bookmark_criteria(entry: &BookmarkEntry, criteria: &MemorySearchCriteria) -> bool {
    // Query matching
    if let Some(query) = &criteria.query {
        let query_lower = query.to_lowercase();
        if !entry.title.to_lowercase().contains(&query_lower)
            && !entry.description.to_lowercase().contains(&query_lower)
            && !entry.url.to_lowercase().contains(&query_lower)
        {
            return false;
        }
    }

    // Tag matching
    if let Some(tags) = &criteria.tags {
        if !tags.iter().any(|tag| entry.tags.contains(tag)) {
            return false;
        }
    }

    // Date range matching
    if let Some((start, end)) = &criteria.date_range {
        if entry.created_at < *start || entry.created_at > *end {
            return false;
        }
    }

    true
}

/// Search notes
pub(super) fn search_notes<'a>(
    memory_data: &'a MemoryData,
    criteria: &'a MemorySearchCriteria,
) -> Vec<(&'a String, &'a NoteEntry)> {
    let mut results: Vec<_> = memory_data
        .notes
        .iter()
        .filter(|(_, entry)| matches_note_criteria(entry, criteria))
        .collect();

    sort_note_results(&mut results, &criteria.sort_by);

    if let Some(limit) = criteria.limit {
        results.truncate(limit);
    }

    results
}

fn matches_note_criteria(entry: &NoteEntry, criteria: &MemorySearchCriteria) -> bool {
    // Query matching
    if let Some(query) = &criteria.query {
        let query_lower = query.to_lowercase();
        if !entry.title.to_lowercase().contains(&query_lower)
            && !entry.content.to_lowercase().contains(&query_lower)
        {
            return false;
        }
    }

    // Category matching
    if let Some(category) = &criteria.category {
        if entry.category != *category {
            return false;
        }
    }

    // Tag matching
    if let Some(tags) = &criteria.tags {
        if !tags.iter().any(|tag| entry.tags.contains(tag)) {
            return false;
        }
    }

    // Date range matching
    if let Some((start, end)) = &criteria.date_range {
        if entry.created_at < *start || entry.created_at > *end {
            return false;
        }
    }

    true
}

fn sort_note_results(results: &mut Vec<(&String, &NoteEntry)>, sort_by: &MemorySortBy) {
    match sort_by {
        MemorySortBy::CreatedAt => results.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at)),
        MemorySortBy::UpdatedAt => results.sort_by(|a, b| b.1.updated_at.cmp(&a.1.updated_at)),
        MemorySortBy::Priority => results.sort_by(|a, b| {
            let a_priority = match a.1.priority {
                NotePriority::Critical => 4,
                NotePriority::High => 3,
                NotePriority::Medium => 2,
                NotePriority::Low => 1,
                NotePriority::Archive => 0,
            };
            let b_priority = match b.1.priority {
                NotePriority::Critical => 4,
                NotePriority::High => 3,
                NotePriority::Medium => 2,
                NotePriority::Low => 1,
                NotePriority::Archive => 0,
            };
            b_priority.cmp(&a_priority)
        }),
        _ => {} // Other sort types
    }
}
