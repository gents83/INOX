use inox_commands::CommandParser;

#[test]
fn parses_commands_and_typed_values() {
    let parser = CommandParser::from_string("-plugin viewer -count 3\n-plugin binarizer");

    assert!(parser.has("plugin"));
    assert_eq!(
        parser.get_values_of::<String>("plugin"),
        vec!["viewer", "binarizer"]
    );
    assert_eq!(parser.get_values_of::<u32>("count"), vec![3]);
}

#[test]
fn unknown_values_use_type_defaults() {
    let parser = CommandParser::from_string("-count not-a-number");

    assert_eq!(parser.get_values_of::<u32>("count"), vec![0]);
    assert!(!parser.has("missing"));
}
