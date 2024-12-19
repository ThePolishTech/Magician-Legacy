pub const REGISTER: &str = "
    INSERT INTO DiscordUsers ( pk_discordId, fk_currentCharacter )
    VALUES ( ?1, null )
";
