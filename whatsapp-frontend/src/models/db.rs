use super::{Contact, Message};
use rusqlite::{Connection, Result, params};
use std::sync::{Arc, Mutex};

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS contacts (
                jid TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                last_message TEXT,
                last_message_time INTEGER,
                unread_count INTEGER DEFAULT 0,
                conversation_timestamp INTEGER DEFAULT 0,
                is_group BOOLEAN DEFAULT 0,
                archived BOOLEAN DEFAULT 0,
                pinned INTEGER DEFAULT 0,
                mute_end_time INTEGER DEFAULT 0,
                profile_picture_url TEXT
            )",
            [],
        )?;

        // Add profile_picture_url column if it doesn't exist (migration)
        let _ = conn.execute(
            "ALTER TABLE contacts ADD COLUMN profile_picture_url TEXT",
            [],
        );

        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                message_id TEXT NOT NULL UNIQUE,
                jid TEXT NOT NULL,
                sender TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                is_from_me BOOLEAN NOT NULL,
                message_type TEXT NOT NULL DEFAULT 'text',
                raw_data TEXT,
                quoted_message_id TEXT,
                media_url TEXT,
                caption TEXT
            )",
            [],
        )?;

        // Migrate old messages table if needed
        let _ = conn.execute("ALTER TABLE messages ADD COLUMN message_id TEXT", []);
        let _ = conn.execute(
            "ALTER TABLE messages ADD COLUMN message_type TEXT DEFAULT 'text'",
            [],
        );
        let _ = conn.execute("ALTER TABLE messages ADD COLUMN raw_data TEXT", []);
        let _ = conn.execute("ALTER TABLE messages ADD COLUMN quoted_message_id TEXT", []);
        let _ = conn.execute("ALTER TABLE messages ADD COLUMN media_url TEXT", []);
        let _ = conn.execute("ALTER TABLE messages ADD COLUMN caption TEXT", []);

        conn.execute(
            "CREATE TABLE IF NOT EXISTS session (
                key TEXT PRIMARY KEY,
                value TEXT
            )",
            [],
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn save_contact(&self, contact: &Contact) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        println!(
            "[DB] Attempting to save contact: {} ({})",
            contact.name, contact.jid
        );
        conn.execute(
            "INSERT OR REPLACE INTO contacts (jid, name, last_message, last_message_time, unread_count, conversation_timestamp, is_group, archived, pinned, mute_end_time, profile_picture_url)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                contact.jid,
                contact.name,
                contact.last_message,
                contact.last_message_time,
                contact.unread_count,
                contact.conversation_timestamp,
                contact.is_group,
                contact.archived,
                contact.pinned,
                contact.mute_end_time,
                contact.profile_picture_url,
            ],
        )?;
        println!("[DB] Contact saved successfully");
        Ok(())
    }

    pub fn get_contacts(&self) -> Result<Vec<Contact>> {
        let conn = self.conn.lock().unwrap();
        println!("[DB] Querying contacts from database...");
        let mut stmt = conn.prepare(
            "SELECT jid, name, last_message, last_message_time, unread_count, conversation_timestamp, is_group, archived, pinned, mute_end_time, profile_picture_url
             FROM contacts 
             ORDER BY conversation_timestamp DESC",
        )?;

        let contacts = stmt
            .query_map([], |row| {
                Ok(Contact {
                    jid: row.get(0)?,
                    name: row.get(1)?,
                    last_message: row.get(2)?,
                    last_message_time: row.get(3)?,
                    unread_count: row.get(4)?,
                    conversation_timestamp: row.get(5)?,
                    is_group: row.get(6)?,
                    archived: row.get(7)?,
                    pinned: row.get(8)?,
                    mute_end_time: row.get(9)?,
                    profile_picture_url: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        println!("[DB] Retrieved {} contacts", contacts.len());
        Ok(contacts)
    }

    pub fn save_message(&self, message: &Message) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        println!(
            "[DB] Saving message: {} in chat {} (type: {})",
            message.message_id, message.jid, message.message_type
        );
        conn.execute(
            "INSERT OR REPLACE INTO messages (message_id, jid, sender, content, timestamp, is_from_me, message_type, raw_data, quoted_message_id, media_url, caption)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                message.message_id,
                message.jid,
                message.sender,
                message.content,
                message.timestamp,
                message.is_from_me,
                message.message_type,
                message.raw_data,
                message.quoted_message_id,
                message.media_url,
                message.caption,
            ],
        )?;
        println!("[DB] Message saved successfully");
        Ok(())
    }

    pub fn get_messages(&self, jid: &str) -> Result<Vec<Message>> {
        let conn = self.conn.lock().unwrap();
        println!("[DB] Querying messages for JID: {}", jid);
        let mut stmt = conn.prepare(
            "SELECT id, message_id, jid, sender, content, timestamp, is_from_me, message_type, raw_data, quoted_message_id, media_url, caption
             FROM messages 
             WHERE jid = ?1 
             ORDER BY timestamp ASC",
        )?;

        let messages = stmt
            .query_map(params![jid], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    message_id: row.get(1)?,
                    jid: row.get(2)?,
                    sender: row.get(3)?,
                    content: row.get(4)?,
                    timestamp: row.get(5)?,
                    is_from_me: row.get(6)?,
                    message_type: row.get(7)?,
                    raw_data: row.get(8)?,
                    quoted_message_id: row.get(9)?,
                    media_url: row.get(10)?,
                    caption: row.get(11)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        println!(
            "[DB] Retrieved {} messages for JID: {}",
            messages.len(),
            jid
        );
        Ok(messages)
    }

    pub fn set_session_data(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO session (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_session_data(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM session WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.get_session_data("authenticated")
            .ok()
            .flatten()
            .map(|v| v == "true")
            .unwrap_or(false)
    }

    pub fn set_authenticated(&self, authenticated: bool) -> Result<()> {
        self.set_session_data(
            "authenticated",
            if authenticated { "true" } else { "false" },
        )
    }
}
