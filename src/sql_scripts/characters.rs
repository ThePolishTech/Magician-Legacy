/// Add a user's character to the database. Auto increments
///
/// Binds:
///   - fk_discordId
///   - pk_name       // Needs to be manually enforced
///   - species
///   - backstory
pub const ADD_CHARACTER: &str = "
    INSERT INTO Characters ( pk_characterId, fk_discordId, pk_name, species, backstory )
    VALUES (
        (SELECT IFNULL(MAX(pk_characterId), 0) + 1 FROM Characters),
        ?1,
        ?2,
        ?3,
        ?4
    )
";

/// Select by owner's discord ID
///
/// Binds:
///   - fk_discordId
pub const SELECT_BY_OWNER_ID: &str = "
    SELECT *
    FROM Characters
    WHERE fk_discordId = ?1;
";

/// Select characters by name and discord user ID
///
/// Binds:
///   - fk_discordId
///   - pk_name
pub const SELECT_BY_NAME_AND_OWNER_ID: &str = "
    SELECT *
    FROM Characters
    WHERE fk_discordId = ?1 AND pk_name = ?2;
";
