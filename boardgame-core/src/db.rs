use rusqlite::{Connection, params};
use std::path::Path;

use crate::errors::Error;

#[derive(Debug)]
pub struct Boardgame {
    pub id: Option<i64>,
    pub name: String,
    pub min_players: i32,
    pub max_players: i32,
    pub play_time_minutes: i32,
    pub description: String,
}

#[derive(Debug)]
pub struct BoardgameDb {
    conn: Connection,
}


impl BoardgameDb {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let conn = Connection::open(path)?;
        
        // Create table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS boardgames (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                min_players INTEGER NOT NULL,
                max_players INTEGER NOT NULL,
                play_time_minutes INTEGER NOT NULL,
                description TEXT NOT NULL
            )",
            [],
        )?;

        Ok(BoardgameDb { conn })
    }

    // Create
    pub fn create(&self, boardgame: &Boardgame) -> Result<i64, Error> {
        self.conn.execute(
            "INSERT INTO boardgames (name, min_players, max_players, play_time_minutes, description)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                boardgame.name,
                boardgame.min_players,
                boardgame.max_players,
                boardgame.play_time_minutes,
                boardgame.description,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    // Read
    pub fn get_all(&self) -> Result<Vec<Boardgame>, Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, min_players, max_players, play_time_minutes, description 
             FROM boardgames"
        )?;

        let boardgames = stmt.query_map([], |row| {
            Ok(Boardgame {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                min_players: row.get(2)?,
                max_players: row.get(3)?,
                play_time_minutes: row.get(4)?,
                description: row.get(5)?,
            })
        })?;

        let res: Result<Vec<Boardgame>, Error> = boardgames.collect::<Result<Vec<Boardgame>, rusqlite::Error>>().map_err(|e| Error::DatabaseError(e));
        Ok(res?)
    }




    pub fn get_by_id(&self, id: i64) -> Result<Option<Boardgame>, Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, min_players, max_players, play_time_minutes, description 
             FROM boardgames WHERE id = ?"
        )?;

        let boardgame = stmt.query_row(params![id], |row| {
            Ok(Boardgame {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                min_players: row.get(2)?,
                max_players: row.get(3)?,
                play_time_minutes: row.get(4)?,
                description: row.get(5)?,
            })
        });

        Ok(match boardgame {
            Ok(game) => Ok(Some(game)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }?)
    }

    // Update
    pub fn update(&self, boardgame: &Boardgame) -> Result<usize, Error> {
        let id = boardgame.id.ok_or(rusqlite::Error::InvalidParameterName("Boardgame must have an id to update".into()))?;

        Ok(self.conn.execute(
            "UPDATE boardgames 
             SET name = ?1, min_players = ?2, max_players = ?3, play_time_minutes = ?4, description = ?5
             WHERE id = ?6",
            params![
                boardgame.name,
                boardgame.min_players,
                boardgame.max_players,
                boardgame.play_time_minutes,
                boardgame.description,
                id,
            ],
        )?)
    }

    // Delete
    pub fn delete(&self, id: i64) -> Result<usize, Error> {
        Ok(self.conn.execute(
            "DELETE FROM boardgames WHERE id = ?",
            params![id],
        )?)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_crud_operations() -> Result<(), Error> {
        let dir = tempdir().expect("failed to create temp directory");
        let db_path = dir.path().join("test.db");
        let db = BoardgameDb::new(db_path)?;

        // Test Create
        let game = Boardgame {
            id: None,
            name: "Catan".to_string(),
            min_players: 3,
            max_players: 4,
            play_time_minutes: 60,
            description: "Resource management and trading game".to_string(),
        };

        let id = db.create(&game)?;
        assert!(id > 0);

        // Test Read
        let retrieved = db.get_by_id(id)?.unwrap();
        assert_eq!(retrieved.name, "Catan");

        // Test Update
        let mut updated_game = retrieved;
        updated_game.name = "Settlers of Catan".to_string();
        db.update(&updated_game)?;

        let retrieved_updated = db.get_by_id(id)?.unwrap();
        assert_eq!(retrieved_updated.name, "Settlers of Catan");

        // Test Delete
        db.delete(id)?;
        assert!(db.get_by_id(id)?.is_none());

        Ok(())
    }
} 