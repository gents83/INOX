use inox_connector::config::Config;
use inox_resources::ConfigBase;
use inox_serialize::{deserialize, serialize, SerializeFile};

#[test]
fn connector_config_has_stable_defaults_and_filename() {
    let config = Config::default();

    assert_eq!(config.host_address, "");
    assert_eq!(config.port, 0);
    assert_eq!(config.get_filename(), "connector.cfg");
    assert_eq!(Config::extension(), "cfg");
}

#[test]
fn connector_config_round_trips_through_binary_serialization() {
    let config = Config {
        host_address: "127.0.0.1".to_owned(),
        port: 4242,
    };

    assert_eq!(deserialize::<Config>(&serialize(&config)), Some(config));
}
