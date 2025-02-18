/// Add a user's character to the database
pub const ADD_CHARACTER: &str = "
    INSERT INTO Characters ( pk_characterId, fk_discordId, name, species, backstory )
    VALUES ( ?1, ?2, ?3, ?4, ?5 )
";

