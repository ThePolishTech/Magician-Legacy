pub const SCRIPT: &str = "
    CREATE TABLE  IF NOT EXISTS    DiscordUsers
    (
        pk_discordId           INTEGER  PRIMARY KEY,
        fk_currentCharacter    INTEGER, -- Can't be NOT NULL due to first registration edge case

        FOREIGN KEY (fk_currentCharacter)
        REFERENCES Characters (pk_CharacterId)
    );

    CREATE TABLE  IF NOT EXISTS    Characters
    (
        pk_CharacterId    INTEGER  PRIMARY KEY,
        fk_discordId      INTEGER  NOT NULL,
        name              TEXT     NOT NULL,
        species           TEXT     NOT NULL,
        backstory         TEXT     NOT NULL,

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
";

