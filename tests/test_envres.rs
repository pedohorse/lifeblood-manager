use lifeblood_manager::config_data_collection::ConfigDataCollection;
use lifeblood_manager::config_data::ConfigData;
use home;

#[test]
fn test_default_location() {
    if let Some(home_path) = home::home_dir() {
        assert!(ConfigDataCollection::default_config_location().starts_with(home_path));
    }
}

#[test]
fn test_validation_syntax() {
    assert!(if let Ok(()) = ConfigData::validate_config_text(
        r#"
[packages.foo.1.2.3]
qet="aj qooq lalala"
env.FOOBAR.append = ["/q/w/e", "/zxc"]
"#
    ) { true } else { false });

    // bad ones
    assert!(if let Err(_) = ConfigData::validate_config_text(
        r#"
[packages.foo.1.2.3]
qet="aj qooq lalala'
env.FOOBAR.append = ["/q/w/e", "/zxc"]
"#
    ) { true } else { false });
    assert!(if let Err(_) = ConfigData::validate_config_text(
        r#"
[packages.foo.1.2.3]
qet="aj qooq lalala"
env.FOO BAR.append = ["/q/w/e", "/zxc"]
"#
    ) { true } else { false });
    assert!(if let Err(_) = ConfigData::validate_config_text(
        r#"
[packages.foo.1.2.3]
qet="aj qooq lalala"
env.FOOBAR.append = ["/q/w/e" "/zxc"]
"#
    ) { true } else { false });
}