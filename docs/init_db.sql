-- Context Database Initialization Script
-- This script sets up the schema for tracking work across sessions

-- Iterations: Named work contexts (e.g., "monorepo-migration", "auth-refactor")
CREATE TABLE IF NOT EXISTS iterations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    last_accessed_at TEXT DEFAULT (datetime('now'))
);

-- Sessions: Your work periods - one per conversation
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    iteration_id INTEGER NOT NULL,
    started_at TEXT DEFAULT (datetime('now')),
    ended_at TEXT,
    summary TEXT,
    FOREIGN KEY (iteration_id) REFERENCES iterations(id)
);

-- Entries: Your running notes AS YOU WORK
CREATE TABLE IF NOT EXISTS entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    entry_type TEXT CHECK(entry_type IN ('progress', 'decision', 'note', 'blocker')) NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- Items: Trackable work units (tasks, features, bugs, phases, milestones)
CREATE TABLE IF NOT EXISTS items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    iteration_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    status TEXT CHECK(status IN ('not_started', 'in_progress', 'blocked', 'complete', 'deferred')) DEFAULT 'not_started',
    priority TEXT CHECK(priority IN ('critical', 'high', 'medium', 'low')) DEFAULT 'medium',
    owner TEXT,
    notes TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    last_session_id INTEGER,
    FOREIGN KEY (iteration_id) REFERENCES iterations(id),
    FOREIGN KEY (last_session_id) REFERENCES sessions(id)
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_sessions_started ON sessions(started_at DESC);
CREATE INDEX IF NOT EXISTS idx_entries_created ON entries(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_items_updated ON items(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_items_iteration_status ON items(iteration_id, status);
CREATE INDEX IF NOT EXISTS idx_items_session ON items(last_session_id);

-- Triggers for automatic timestamp updates
CREATE TRIGGER IF NOT EXISTS update_items_timestamp
AFTER UPDATE ON items
WHEN NEW.title != OLD.title OR NEW.status != OLD.status OR NEW.priority != OLD.priority
BEGIN
    UPDATE items SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_iteration_accessed
AFTER UPDATE OF last_accessed_at ON iterations
BEGIN
    UPDATE iterations SET last_accessed_at = datetime('now') WHERE id = NEW.id;
END;
