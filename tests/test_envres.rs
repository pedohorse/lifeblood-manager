use lifeblood_manager::config_data_collection::ConfigDataCollection;
use lifeblood_manager::config_data::{ConfigData, ConfigError};
use home;

#[test]
fn test_default_location() {
    if let Some(home_path) = home::home_dir() {
        assert!(ConfigDataCollection::default_config_location().starts_with(home_path));
    }
}

#[test]
fn test_validation_syntax() {
    //
    // good ones
    assert!(if let Ok(()) = ConfigData::validate_config_text(
        r#"
[packages.foo."1.2.3"]
qet="aj qooq lalala"
env.FOOBAR.append = ["/q/w/e", "/zxc"]
"#
    ) { true } else { false });

    //
    // bad ones
    assert!(if let Err(ConfigError::SyntaxError(_, _)) = ConfigData::validate_config_text(
        r#"
[packages.foo."1.2.3"]
qet="aj qooq lalala'
env.FOOBAR.append = ["/q/w/e", "/zxc"]
"#
    ) { true } else { false });
    assert!(if let Err(ConfigError::SyntaxError(_, _)) = ConfigData::validate_config_text(
        r#"
[packages.foo."1.2.3"]
qet="aj qooq lalala"
env.FOO BAR.append = ["/q/w/e", "/zxc"]
"#
    ) { true } else { false });
    assert!(if let Err(ConfigError::SyntaxError(_, _)) = ConfigData::validate_config_text(
        r#"
[packages.foo."1.2.3"]
qet="aj qooq lalala"
env.FOOBAR.append = ["/q/w/e" "/zxc"]
"#
    ) { true } else { false });
}

#[test]
fn test_validation_schema1() {
    //
    // good ones
    assert!(if let Ok(()) = ConfigData::validate_config_text(
        r#"
[packages.foo."1.2.3"]
qet="aj qooq lalala"
env.FOOBAR.append = ["/q/w/e", "/zxc"]
"#
    ) { true } else { false });

    //
    // bad ones
    assert!(if let Err(ConfigError::SchemaError(_)) = ConfigData::validate_config_text(
        r#"
packages=1.2
"#
    ) { true } else { false });
    assert!(if let Err(ConfigError::SchemaError(_)) = ConfigData::validate_config_text(
        r#"
packages.foo = 2
"#
    ) { true } else { false });
    assert!(if let Err(ConfigError::SchemaError(_)) = ConfigData::validate_config_text(
        r#"
[packages.foo.1.2.3]
qet="aj qooq lalala"
env.FOOBAR.append = ["/q/w/e", "/zxc"]
"#
    ) { true } else { false });
    assert!(if let Err(ConfigError::SchemaError(_)) = ConfigData::validate_config_text(
        r#"
[packages.foo."1.2"]
qet="aj qooq lalala"
env.FOOBAR.append = ["/q/w/e", "/zxc"]
"#
    ) { true } else { false });
    assert!(if let Err(ConfigError::SchemaError(_)) = ConfigData::validate_config_text(
        r#"
[packages.foo."1.2.3.4"]
qet="aj qooq lalala"
env.FOOBAR.append = ["/q/w/e", "/zxc"]
"#
    ) { true } else { false });
    assert!(if let Err(ConfigError::SchemaError(_)) = ConfigData::validate_config_text(
        r#"
[packages.foo."1.2.q"]
qet="aj qooq lalala"
env.FOOBAR.append = ["/q/w/e", "/zxc"]
"#
    ) { true } else { false });
    assert!(if let Err(ConfigError::SchemaError(_)) = ConfigData::validate_config_text(
        r#"
[packages.foo."1.2..3"]
qet="aj qooq lalala"
env.FOOBAR.append = ["/q/w/e", "/zxc"]
"#
    ) { true } else { false });
}