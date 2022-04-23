CREATE TABLE `tag` (
    `tag`           TEXT
);

CREATE TABLE `word` (
    `word`          TEXT
);
CREATE TABLE `word_tag` (
    `word`          INTEGER,
    `tag`           INTEGER,
    FOREIGN KEY (`word`)            REFERENCES `word`(`rowid`),
    FOREIGN KEY (`tag`)             REFERENCES `tag`(`rowid`)
);
CREATE TABLE `word_reading` (
    `word`          INTEGER,
    `full`          TEXT,
    `accent`        TEXT,
    FOREIGN KEY (`word`)            REFERENCES `word`(`rowid`)
);
CREATE TABLE `definition` (
    `word_reading`  INTEGER,
    `definition`    TEXT,
    FOREIGN KEY (`word_reading`)    REFERENCES `word_reading`(`rowid`)
);
CREATE TABLE `kanji` (
    `kanji`         CHAR(1),
    `memonic`       TEXT
);
CREATE TABLE `kanji_tag` (
    `kanji`         INTEGER,
    `tag`           INTEGER,
    FOREIGN KEY (`kanji`)           REFERENCES `kanji`(`rowid`),
    FOREIGN KEY (`tag`)             REFERENCES `tag`(`rowid`)
);
CREATE TABLE `kanji_reading` (
    `kanji`         INTEGER,
    `memonic`       TEXT,
    FOREIGN KEY (`kanji`)           REFERENCES `kanji`(`rowid`)
);
CREATE TABLE `reading` (
    /* The word/kanji readings */
    `word`          INTEGER,
    `kanji`         INTEGER,
    /* kanji character index in the word */
    `index`         INTEGER,
    FOREIGN KEY (`word`)            REFERENCES `word_reading`(`rowid`),
    FOREIGN KEY (`kanji`)           REFERENCES `kanji_reading`(`rowid`)
);

CREATE TABLE `sentence` (
    `sentence`      TEXT,
    `translation`   TEXT
);
CREATE TABLE `resource` (
    `name`          TEXT, 
    `kind`          TEXT,
    `uri`           TEXT
);