use super::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Word {
    pub word: String,
    pub readings: Vec<WordReading>,
    pub tags: Vec<data::Tag>
}
impl Word {
    pub fn get(c: &Connection, word: &str) -> Result<Option<Self>> {
        Ok(c.prepare("SELECT `word`, `rowid` FROM `word` WHERE `word` = ?1")?
            .query_row([word], |row| {
                let word_id = row.get(1)?;
                Ok(Self {
                    word: row.get(0)?,
                    readings: WordReading::for_word(c, word_id)?,
                    tags: data::Tag::for_word(c, word_id)?
                })
            })
            .optional()?
        )
    }
    pub fn insert(self, c: &Connection) -> Result<()> {
        let word_id = c.prepare("INSERT INTO `word` (`word`) VALUES (?1)")?
            .insert([&self.word])?;
        for reading in self.readings {
            
        }
        for tag in self.tags {
            let tag_id = tag.get_or_insert(c)?;
            c.prepare("INSERT INTO `word_tag` (`word`, `tag`) VALUES (?1, ?2)")?
                .execute([word_id, tag_id])?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WordReading {
    pub full: String,
    pub accent: String,
    pub definitions: Vec<Definition>
}
impl WordReading {
    const QUERY: &'static str = "SELECT `full`, `accent`, `rowid` FROM `word_reading` WHERE `word` = ?1";
    pub fn for_word(c: &Connection, word: u64) -> rusqlite::Result<Vec<Self>> {
        let mut s = c.prepare(Self::QUERY)?;
        let rows = s.query_map([word], |row| {
                Ok(Self {
                    full: row.get(0)?,
                    accent: row.get(1)?,
                    definitions: Definition::for_reading(c, row.get(2)?)?
                })
            })?;
        let mut readings = Vec::new();
        for reading in rows {
            readings.push(reading?)
        }
        Ok(readings)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Definition {
    pub definition: String,
}
impl std::fmt::Display for Definition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.definition)
    }
}
impl Definition {
    const QUERY: &'static str = "SELECT `definition` FROM `definition` WHERE `definition`.`word_reading` = ?1";
    pub fn for_reading(c: &Connection, reading: u64) -> rusqlite::Result<Vec<Self>> {
        let mut s = c.prepare(Self::QUERY)?;
        let rows = s.query_map([reading], |row| {
                Ok(Self {
                    definition: row.get(0)?
                })
            })?;
        let mut definitions = Vec::new();
        for definition in rows {
            definitions.push(definition?)
        }
        Ok(definitions)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Kanji {
    pub kanji: char,
    pub memonic: String
}
impl Kanji {
    const QUERY: &'static str = "SELECT `kanji`, `memonic` FROM `kanji` WHERE `kanji` = ?1";
    pub fn get(c: &Connection, kanji: char) -> rusqlite::Result<Option<Self>> {
        Ok(c.prepare(Self::QUERY)?.query_row([kanji.to_string()], |row| {
            Ok(Self {
                kanji: row.get::<_, String>(0)?.chars().next().ok_or(rusqlite::Error::QueryReturnedNoRows)?,
                memonic: row.get(1)?
            })
        }).optional()?)
    }
    /*pub fn for_reading(c: &Connection, reading: u64) -> rusqlite::Result<Vec<Self>> {
        let mut s = c.prepare(Self::QUERY)?;
        let rows = s.query_map([reading], |row| {
                Ok(Self {
                    definition: row.get(0)?
                })
            })?;
        let mut definitions = Vec::new();
        for definition in rows {
            definitions.push(definition?)
        }
        Ok(definitions)
    }*/
}