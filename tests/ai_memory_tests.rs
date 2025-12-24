// AI Memory Heap comprehensive test suite
// Tests for research, credentials, sessions, bookmarks, notes, and utilities

mod features {
    pub mod ai_memory {
        include!("features/ai_memory/research_tests.rs");
    }
}

mod credentials {
    include!("features/ai_memory/credentials_tests.rs");
}

mod sessions {
    include!("features/ai_memory/sessions_tests.rs");
}

mod bookmarks {
    include!("features/ai_memory/bookmarks_tests.rs");
}

mod notes {
    include!("features/ai_memory/notes_tests.rs");
}

mod utility {
    include!("features/ai_memory/utility_tests.rs");
}
