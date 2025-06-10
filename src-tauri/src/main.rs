// src-tauri/src/main.rs

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs, path::Path, sync::Mutex};
use rusqlite::{Connection, Result};
use tauri::State;
use walkdir::WalkDir;
use std::time::UNIX_EPOCH;
use tauri::Manager;

// Database wrapper struct
struct DbConnection(Mutex<Connection>);

#[derive(Debug, serde::Serialize)]
struct FileMeta {
    name: String,
    path: String,
    extension: Option<String>,
    size: u64,
    modified: u64,
}

// Error handling
#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    Rusqlite(rusqlite::Error),
    MutexPoison,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Error::Rusqlite(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Rusqlite(e) => write!(f, "Database error: {}", e),
            Error::MutexPoison => write!(f, "Mutex poisoned"),
        }
    }
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize database
            let app_dir = app.path_resolver().app_data_dir().unwrap();
            println!("Creating database in: {:?}", app_dir);
            std::fs::create_dir_all(&app_dir)?;
            let db_path = app_dir.join("file_explorer.sqlite3");
            println!("Database path: {:?}", db_path);
            
            let conn = Connection::open(&db_path)?;
            println!("Database connection established");
            create_table(&conn)?;
            create_indexes(&conn)?;
            println!("Database tables and indexes created");
            
            app.manage(DbConnection(Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_children,
            search_files,
            get_file_meta_command,
            transfer_to_sqlite,
            get_directory_size,
            database_has_files,
            list_directory_contents,
            open_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn list_children(db: State<DbConnection>, dir: String) -> Result<Vec<FileMeta>, Error> {
    let conn = db.0.lock().map_err(|_| Error::MutexPoison)?;
    let norm_dir = dir.trim_end_matches('\\');
    let slash_count = norm_dir.matches('\\').count();
    let target_slash_count = slash_count + 1;

    let like_pattern = format!("{}\\%", norm_dir.replace("\\", "\\\\"));

    let mut stmt = conn.prepare(
        "SELECT name, path, extension, size, modified
         FROM main_table
         WHERE path LIKE ?1 ESCAPE '\\'
         AND (LENGTH(path) - LENGTH(REPLACE(path, '\\', ''))) = ?2",
    )?;

    let rows = stmt.query_map(rusqlite::params![like_pattern, target_slash_count], |row| {
        Ok(FileMeta {
            name: row.get(0)?,
            path: row.get(1)?,
            extension: row.get(2)?,
            size: row.get(3)?,
            modified: row.get(4)?,
        })
    })?;

    Ok(rows.filter_map(Result::ok).collect())
}

#[tauri::command]
fn search_files(
    db: State<DbConnection>,
    name: String,
    extension: String,
) -> Result<Vec<FileMeta>, Error> {
    let conn = db.0.lock().map_err(|_| Error::MutexPoison)?;

    fn map_row(row: &rusqlite::Row) -> Result<FileMeta, rusqlite::Error> {
        Ok(FileMeta {
            name: row.get(0)?,
            path: row.get(1)?,
            extension: row.get(2)?,
            size: row.get(3)?,
            modified: row.get(4)?,
        })
    }

    let result = if !extension.is_empty() {
        let mut stmt = conn.prepare(
            "SELECT name, path, extension, size, modified
             FROM main_table
             WHERE name LIKE ?1 AND extension = ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![&format!("%{}%", name), &extension], map_row)?;
        rows.filter_map(Result::ok).collect::<Vec<_>>()
    } else {
        let mut stmt = conn.prepare(
            "SELECT name, path, extension, size, modified
             FROM main_table
             WHERE name LIKE ?1",
        )?;
        let rows = stmt.query_map(rusqlite::params![&format!("%{}%", name)], map_row)?;
        rows.filter_map(Result::ok).collect::<Vec<_>>()
    };
    Ok(result)
}

#[tauri::command]
fn get_file_meta_command(path: String) -> Result<FileMeta, Error> {
    get_file_meta(Path::new(&path)).map_err(Into::into)
}

#[tauri::command]
fn database_has_files(db: State<DbConnection>) -> Result<bool, Error> {
    let conn = db.0.lock().map_err(|_| Error::MutexPoison)?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM main_table",
        [],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

#[tauri::command]
fn transfer_to_sqlite(db: State<DbConnection>, path: String) -> Result<(), Error> {
    let mut conn = db.0.lock().map_err(|_| Error::MutexPoison)?;
    let tx = conn.transaction()?;
    let skip_keywords = ["CloudStore", "OneDrive", "System Volume Information"];

    for entry in WalkDir::new(&path)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_symlink())
        .filter(|e| {
            let path_str = e.path().display().to_string();
            !skip_keywords.iter().any(|k| path_str.contains(k))
        })
    {
        match get_file_meta(entry.path()) {
            Ok(file_meta) => {
                if let Err(err) = insert_file_meta(&tx, &file_meta) {
                    eprintln!("DB insert error for {:?}: {:?}", entry.path(), err);
                }
            }
            Err(_) => {
                // silently skip
            }
        }
    }

    tx.commit()?;
    Ok(())
}

#[tauri::command]
fn get_directory_size(db: State<DbConnection>, path: String) -> Result<u64, Error> {
    let conn = db.0.lock().map_err(|_| Error::MutexPoison)?;
    let mut stmt = conn.prepare(
        "SELECT COALESCE(SUM(size), 0) FROM main_table WHERE path LIKE ?1 || '%'",
    )?;
    
    let size: u64 = stmt.query_row(rusqlite::params![&path], |row| row.get(0))?;
    Ok(size)
}

#[tauri::command]
fn list_directory_contents(path: String) -> Result<Vec<FileMeta>, Error> {
    let dir = Path::new(&path);
    let mut contents = Vec::new();
    
    for entry in fs::read_dir(dir).map_err(Error::Io)? {
        match entry {
            Ok(entry) => {
                if let Ok(meta) = get_file_meta(entry.path().as_ref()) {
                    contents.push(meta);
                }
            }
            Err(e) => eprintln!("Error reading directory entry: {}", e),
        }
    }
    
    Ok(contents)
}

#[tauri::command]
fn open_file(path: String) -> Result<(), Error> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("cmd")
            .args(&["/C", "start", "", &path])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .map_err(|e| Error::Io(e))?;
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        opener::open(path).map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    }
    
    Ok(())
}

fn get_file_meta(path: &Path) -> std::io::Result<FileMeta> {
    let metadata = fs::metadata(path)?;

    let modified = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_string());

    Ok(FileMeta {
        name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        path: path.to_string_lossy().to_string(),
        extension,
        size: metadata.len(),
        modified,
    })
}

fn create_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS main_table (
            name TEXT NOT NULL,
            path TEXT UNIQUE NOT NULL,
            extension TEXT,
            size INTEGER NOT NULL,
            modified INTEGER NOT NULL
        )",
        [],
    )?;
    Ok(())
}

fn create_indexes(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_path ON main_table(path)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_name ON main_table(name)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_extension ON main_table(extension)",
        [],
    )?;
    Ok(())
}

fn insert_file_meta(conn: &Connection, file: &FileMeta) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO main_table (name, path, extension, size, modified)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            file.name,
            file.path,
            file.extension,
            file.size,
            file.modified
        ],
    )?;
    Ok(())
}