CREATE TABLE  IF NOT EXISTS    DiscordUsers
(
    pk_discordId           INTEGER  PRIMARY KEY,
    fk_currentCharacter    INTEGER, -- Can be NOT NULL due to first registration edge case

    FOREIGN KEY (fk_currentCharacter)
    REFERENCES Characters (pk_characterId)
);

CREATE TABLE  IF NOT EXISTS    Characters
(
    pk_characterId    INTEGER  PRIMARY KEY,
    fk_discordId      INTEGER  NOT NULL,
    pk_name           TEXT     NOT NULL,  -- Will be manually enforced in code as a FOREIGN
    species           TEXT     NOT NULL,     -- KEY can't reference only part of a composite
    backstory         TEXT     NOT NULL,     -- PRIMARY KEY

    FOREIGN KEY (fk_discordId)
    REFERENCES DiscordUsers (pk_discordId)
);

CREATE TABLE  IF NOT EXISTS    Atributes
(
    fk_pk_characterId    INTEGER  PRIMARY KEY,

    Strength             INTEGER  NOT NULL,
    Dexterity            INTEGER  NOT NULL,
    Preception           INTEGER  NOT NULL,

    Knowledge            INTEGER  NOT NULL,
    Constitution         INTEGER  NOT NULL,
    Casting              INTEGER  NOT NULL,

    FOREIGN KEY (fk_pk_characterId)
    REFERENCES Characters (pk_CharacterId)
);

CREATE TABLE  IF NOT EXISTS    CharacterAbilities
(
    fk_pk_characterId     INTEGER  NOT NULL,
    pk_abilityId          INTEGER  NOT NULL,
    abilityName           TEXT     NOT NULL,
    abilityDescription    TEXT     NOT NULL,

    FOREIGN KEY (fk_pk_characterId)
    REFERENCES Characters (pk_characterId),
    PRIMARY KEY (fk_pk_characterId, pk_abilityId)
);

