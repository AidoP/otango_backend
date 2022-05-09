CREATE TABLE `user` (
    `name`          TEXT,
    `contact`       TEXT,
    `image`         BLOB,
    `privilege`     INTEGER,
    `pubkey`        TEXT
);
CREATE UNIQUE INDEX `idx_user_name`
    ON `user`(`name`);

CREATE TABLE `challenge` (
    `user`          INTEGER,
    `challenge`     TEXT,
    `expires`       DATETIME,
    FOREIGN KEY (`user`)            REFERENCES `user`(`rowid`)
);
CREATE UNIQUE INDEX `idx_challenge`
    ON `challenge`(`challenge`);

CREATE TABLE `tag` (
    `tag`           TEXT
);
CREATE UNIQUE INDEX `idx_tag`
    ON `tag`(`tag`);

CREATE TABLE `word` (
    `word`          TEXT
);
CREATE UNIQUE INDEX `idx_word`
    ON `word`(`word`);
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
CREATE UNIQUE INDEX `idx_kanji`
    ON `kanji`(`kanji`);
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