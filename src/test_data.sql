INSERT INTO `word`
    (`rowid`,   `word`) 
VALUES
    (0,         '単語'),
    (1,         '誕生日'),
    (2,         '日本'),
    (3,         '漢字');

INSERT INTO `word_reading`
    (`rowid`,   `word`,     `full`,         `accent`) 
VALUES
    (0,         0,          'たんご',       'TODO'),
    (1,         1,          'たんじょうび', 'TODO'),
    (2,         2,          'にほん',       'TODO'),
    (3,         2,          'にっぽん',     'TODO');

INSERT INTO `definition`
    (`word_reading`,    `definition`) 
VALUES
    (0,          'word'),
    (1,          'birthday'),
    (2,          'Japan'),
    (2,          'Land of the Rising Sun'),
    (3,          'Japan'),
    (3,          'Land of the Rising Sun');

INSERT INTO `kanji`
    (`rowid`,   `kanji`, `memonic`)
VALUES
    (0,         '日', 'sun'),
    (1,         '本', 'origin'),
    (2,         '語', 'language'),
    (3,         '字', 'character');

/*

INSERT INTO `reading`
    (`word`,    `kanji`,   `from`, `to`,   `reading`)
VALUES
    (1,         3,          2,      5,     'ご');

INSERT INTO `sentence`
    (`sentence`,                            `translation`)
VALUES
    ('日本へ行きましょう',                      "Let's go to Japan!");

INSERT INTO `resource`
    (`kind`,    `name`,             `uri`)
VALUES
    ('audio',   '誕生日_male',      '/static/audio/male/誕生日.m4a');
*/