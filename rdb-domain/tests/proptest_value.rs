use std::borrow::Cow;

use proptest::prelude::*;
use rdb_domain::Value;

fn arb_value() -> impl Strategy<Value = Value<'static>> {
  let null = Just(Value::Null);
  let integer = any::<i64>().prop_map(Value::Integer);

  // 避免 NaN/Inf (否则 PartialEq roundtrip 会失败)
  let real = proptest::num::f64::ANY
    .prop_filter("finite f64", |f| f.is_finite())
    .prop_map(Value::Real);

  let text = proptest::collection::vec(any::<char>(), 0..64).prop_map(|chars| {
    let s: String = chars.into_iter().collect();
    Value::Text(Cow::Owned(s))
  });

  let blob =
    proptest::collection::vec(any::<u8>(), 0..256).prop_map(|bytes| Value::Blob(Cow::Owned(bytes)));

  prop_oneof![null, integer, real, text, blob]
}

proptest! {
  #[test]
  fn value_serde_roundtrip_bincode(v in arb_value()) {
    let bytes = bincode::serialize(&v).unwrap();

    // 注意：不能直接反序列化到 Value<'static>,先反到 Value<'_> 再 into_owned()
    let de: Value<'_> = bincode::deserialize(&bytes).unwrap();
    let v2 = de.into_owned();

    prop_assert_eq!(v, v2);
  }
}
