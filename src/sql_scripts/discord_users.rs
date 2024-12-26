pub const REGISTER: &str = "
    INSERT INTO DiscordUsers ( pk_discordId, fk_currentCharacter )
    VALUES ( ?1, null )
";

pub const SELECT_BY_ID: &str = "
    SELECT *
    FROM DiscordUsers
    WHERE pk_discordId = ?1
";

pub const REMOVE_ENTRY: &str = "
    DELETE FROM DiscordUsers
    WHERE pk_discordId = ?1
";

