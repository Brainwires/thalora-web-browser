use super::crypto::{decrypt_password, encrypt_password};
use super::types::*;
use anyhow::Result;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;

/// Store research findings
pub(super) fn store_research(
    memory_data: &mut MemoryData,
    key: &str,
    research: ResearchEntry,
) -> Result<()> {
    memory_data.research.insert(key.to_string(), research);
    Ok(())
}

/// Get research entry by key
pub(super) fn get_research<'a>(memory_data: &'a MemoryData, key: &str) -> Option<&'a ResearchEntry> {
    memory_data.research.get(key)
}

/// Update research entry
pub(super) fn update_research<F>(
    memory_data: &mut MemoryData,
    key: &str,
    updater: F,
) -> Result<bool>
where
    F: FnOnce(&mut ResearchEntry),
{
    if let Some(research) = memory_data.research.get_mut(key) {
        research.updated_at = Utc::now();
        updater(research);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Store credentials (password will be encrypted)
pub(super) fn store_credentials(
    memory_data: &mut MemoryData,
    key: &str,
    service: &str,
    username: &str,
    password: &str,
    additional_data: HashMap<String, String>,
) -> Result<()> {
    let encrypted_password = encrypt_password(password)?;

    let credential = CredentialEntry {
        service: service.to_string(),
        username: username.to_string(),
        encrypted_password,
        additional_data,
        created_at: Utc::now(),
        last_used: None,
        tags: vec![],
    };

    memory_data.credentials.insert(key.to_string(), credential);
    Ok(())
}

/// Get credentials (password will be decrypted)
pub(super) fn get_credentials(
    memory_data: &mut MemoryData,
    key: &str,
) -> Result<Option<(String, String, String, HashMap<String, String>)>> {
    if let Some(cred) = memory_data.credentials.get(key) {
        let encrypted_password = cred.encrypted_password.clone();
        let password = decrypt_password(&encrypted_password)?;
        let result = Some((
            cred.service.clone(),
            cred.username.clone(),
            password,
            cred.additional_data.clone(),
        ));

        // Update last_used timestamp
        if let Some(cred_mut) = memory_data.credentials.get_mut(key) {
            cred_mut.last_used = Some(Utc::now());
        }

        Ok(result)
    } else {
        Ok(None)
    }
}

/// List all stored credential keys (without exposing passwords)
pub(super) fn list_credential_keys(memory_data: &MemoryData) -> Vec<(&String, &str, &str)> {
    memory_data
        .credentials
        .iter()
        .map(|(key, cred)| (key, cred.service.as_str(), cred.username.as_str()))
        .collect()
}

/// Start new session
pub(super) fn start_session(
    memory_data: &mut MemoryData,
    session_id: &str,
    context: &str,
    objectives: Vec<String>,
) -> Result<()> {
    let session = SessionData {
        session_id: session_id.to_string(),
        context: context.to_string(),
        progress: HashMap::new(),
        objectives,
        completed_tasks: vec![],
        created_at: Utc::now(),
        last_activity: Utc::now(),
        status: SessionStatus::Active,
    };

    memory_data
        .sessions
        .insert(session_id.to_string(), session);
    Ok(())
}

/// Update session progress
pub(super) fn update_session_progress(
    memory_data: &mut MemoryData,
    session_id: &str,
    key: &str,
    value: Value,
) -> Result<bool> {
    if let Some(session) = memory_data.sessions.get_mut(session_id) {
        session.progress.insert(key.to_string(), value);
        session.last_activity = Utc::now();
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Mark task as completed in session
pub(super) fn complete_session_task(
    memory_data: &mut MemoryData,
    session_id: &str,
    task: &str,
) -> Result<bool> {
    if let Some(session) = memory_data.sessions.get_mut(session_id) {
        session.completed_tasks.push(task.to_string());
        session.last_activity = Utc::now();
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Get active sessions
pub(super) fn get_active_sessions(memory_data: &MemoryData) -> Vec<(&String, &SessionData)> {
    memory_data
        .sessions
        .iter()
        .filter(|(_, session)| matches!(session.status, SessionStatus::Active))
        .collect()
}

/// Store bookmark
pub(super) fn store_bookmark(
    memory_data: &mut MemoryData,
    key: &str,
    url: &str,
    title: &str,
    description: &str,
    content_preview: &str,
    tags: Vec<String>,
) -> Result<()> {
    let bookmark = BookmarkEntry {
        url: url.to_string(),
        title: title.to_string(),
        description: description.to_string(),
        content_preview: content_preview.to_string(),
        tags,
        created_at: Utc::now(),
        last_accessed: None,
        access_count: 0,
        importance_score: 0.5,
    };

    memory_data.bookmarks.insert(key.to_string(), bookmark);
    Ok(())
}

/// Access bookmark (increments counter)
/// SECURITY: Uses saturating_add to prevent integer overflow (CWE-190)
pub(super) fn access_bookmark(memory_data: &mut MemoryData, key: &str) -> Option<BookmarkEntry> {
    if let Some(bookmark) = memory_data.bookmarks.get_mut(key) {
        bookmark.last_accessed = Some(Utc::now());
        // Use saturating_add to prevent overflow - will cap at u64::MAX
        bookmark.access_count = bookmark.access_count.saturating_add(1);
        Some(bookmark.clone())
    } else {
        None
    }
}

/// Store note
pub(super) fn store_note(
    memory_data: &mut MemoryData,
    key: &str,
    title: &str,
    content: &str,
    category: &str,
    tags: Vec<String>,
    priority: NotePriority,
) -> Result<()> {
    let note = NoteEntry {
        title: title.to_string(),
        content: content.to_string(),
        category: category.to_string(),
        tags,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        priority,
        related_entries: vec![],
    };

    memory_data.notes.insert(key.to_string(), note);
    Ok(())
}

/// Update note
pub(super) fn update_note<F>(memory_data: &mut MemoryData, key: &str, updater: F) -> Result<bool>
where
    F: FnOnce(&mut NoteEntry),
{
    if let Some(note) = memory_data.notes.get_mut(key) {
        note.updated_at = Utc::now();
        updater(note);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Clean up old entries based on age and usage
pub(super) fn cleanup_old_entries(
    memory_data: &mut MemoryData,
    max_age_days: i64,
) -> Result<usize> {
    let cutoff_date = Utc::now() - chrono::Duration::days(max_age_days);
    let mut removed_count = 0;

    // Clean up old sessions
    let old_sessions: Vec<String> = memory_data
        .sessions
        .iter()
        .filter(|(_, session)| {
            session.last_activity < cutoff_date
                && !matches!(session.status, SessionStatus::Active)
        })
        .map(|(key, _)| key.clone())
        .collect();

    for key in old_sessions {
        memory_data.sessions.remove(&key);
        removed_count += 1;
    }

    // Clean up low-importance bookmarks
    let old_bookmarks: Vec<String> = memory_data
        .bookmarks
        .iter()
        .filter(|(_, bookmark)| {
            bookmark.created_at < cutoff_date
                && bookmark.access_count == 0
                && bookmark.importance_score < 0.3
        })
        .map(|(key, _)| key.clone())
        .collect();

    for key in old_bookmarks {
        memory_data.bookmarks.remove(&key);
        removed_count += 1;
    }

    // Clean up archived notes
    let old_notes: Vec<String> = memory_data
        .notes
        .iter()
        .filter(|(_, note)| {
            matches!(note.priority, NotePriority::Archive) && note.updated_at < cutoff_date
        })
        .map(|(key, _)| key.clone())
        .collect();

    for key in old_notes {
        memory_data.notes.remove(&key);
        removed_count += 1;
    }

    if removed_count > 0 {
        memory_data.metadata.last_cleanup = Some(Utc::now());
    }

    Ok(removed_count)
}
