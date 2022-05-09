use super::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub tag: String,
}
impl Tag {
    /// Get the rowid of a given tag
    pub fn id(&self, c: &Connection) -> Result<i64> {
        c.prepare("SELECT `rowid` FROM `tag` WHERE `tag`.`tag` = ?1")?
            .query_row([&self.tag], |row| row.get::<_, i64>(0))
            .map_err(|e| e.into())
    }
    pub fn for_word(c: &Connection, word: u64) -> rusqlite::Result<Vec<Self>> {
        let mut s = c.prepare("SELECT `tag`.`tag` FROM `word_tag` INNER JOIN `tag` ON `word_tag`.`tag` = `tag`.`rowid` WHERE `word_tag`.`word` = ?1")?;
        let rows = s.query_map([word], |row| {
                Ok(Self {
                    tag: row.get(0)?
                })
            })?;
        let mut tags = Vec::new();
        for tag in rows {
            tags.push(tag?)
        }
        Ok(tags)
    }
    /// Insert the tag to the database if it doesn't exist, then return the rowid
    pub fn get_or_insert(&self, c: &Connection) -> Result<i64> {
        // Get first as most of the time the tag will already exist
        match self.id(c) {
            Ok(id) => Ok(id),
            Err(_) => match c.prepare("INSERT OR IGNORE INTO `tag` (`tag`) VALUES (?1)")?.execute([&self.tag]) {
                Ok(1) => Ok(c.last_insert_rowid()),
                Ok(0) => self.id(c), // The tag must have been inserted in the mean time
                Ok(_) => unreachable!(/* Only one row can be inserted */),
                Err(e) => Err(e.into())
            }
        }
    }
}