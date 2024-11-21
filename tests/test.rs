#[cfg(test)]
mod tests {
    use super::*;
    use config::Config;

    #[test]
    fn test_config_parsing() {
        let settings = Config::builder()
            .add_source(config::File::from_str(
                r#"
                [api]
                key = "test_api_key"
                waitlist_id = "test_waitlist_id"
                "#,
                config::FileFormat::Toml,
            ))
            .build()
            .expect("Failed to parse configuration");

        let api_key: String = settings.get("api.key").unwrap();
        let waitlist_id: String = settings.get("api.waitlist_id").unwrap();

        assert_eq!(api_key, "test_api_key");
        assert_eq!(waitlist_id, "test_waitlist_id");
    }
}
