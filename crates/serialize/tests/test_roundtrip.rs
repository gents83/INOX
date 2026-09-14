use inox_serialize::{
    deserialize, deserialize_from_text, serialize, serialize_to_text, Deserialize, Serialize,
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TestData {
    name: String,
    values: Vec<u32>,
}

fn sample() -> TestData {
    TestData {
        name: "inox".to_owned(),
        values: vec![1, 2, 3],
    }
}

#[test]
fn round_trips_binary_flexbuffers_data() {
    let value = sample();
    let bytes = serialize(&value);

    assert_eq!(deserialize::<TestData>(&bytes), Some(value));
}

#[test]
fn round_trips_json_data_and_rejects_invalid_input() {
    let value = sample();
    let bytes = serialize_to_text(&value);

    assert_eq!(deserialize_from_text::<TestData>(&bytes), Some(value));
    assert_eq!(deserialize_from_text::<TestData>(b"not json"), None);
}
