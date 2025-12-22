//! 领域模型单元测试
//!
//! 测试所有领域类型的构造和验证功能

use rdb_domain::*;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::path::Path;

// ===============================================
// ID 类型测试
// ===============================================

#[test]
fn test_table_id() {
  let id = TableId::new(1);
  assert_eq!(id.into_inner(), 1);
  assert_eq!(u32::from(id), 1);
  assert_eq!(TableId::from(2), TableId::new(2));
}

#[test]
fn test_column_id() {
  let id = ColumnId::new(1);
  assert_eq!(id.into_inner(), 1);
  assert_eq!(u32::from(id), 1);
  assert_eq!(ColumnId::from(2), ColumnId::new(2));
}

#[test]
fn test_row_id() {
  let id = RowId::new(1);
  assert_eq!(id.into_inner(), 1);
  assert_eq!(i64::from(id), 1);
  assert_eq!(RowId::from(2), RowId::new(2));
}

#[test]
fn test_page_id() {
  let id = PageId::new(1);
  assert_eq!(id.into_inner(), 1);
  assert_eq!(u32::from(id), 1);
  assert_eq!(PageId::from(2), PageId::new(2));
}

#[test]
fn test_index_id() {
  let id = IndexId::new(1);
  assert_eq!(id.into_inner(), 1);
  assert_eq!(u32::from(id), 1);
  assert_eq!(IndexId::from(2), IndexId::new(2));
}

#[test]
fn test_transaction_id() {
  let id = TransactionId::new(1);
  assert_eq!(id.into_inner(), 1);
  assert_eq!(u64::from(id), 1);
  assert_eq!(TransactionId::from(2), TransactionId::new(2));
}

#[test]
fn test_lock_id() {
  let id = LockId::new(1);
  assert_eq!(id.into_inner(), 1);
  assert_eq!(u64::from(id), 1);
  assert_eq!(LockId::from(2), LockId::new(2));
}

// ===============================================
// DataType 测试
// ===============================================

#[test]
fn test_data_type_variants() {
  assert_eq!(DataType::Integer, DataType::Integer);
  assert_eq!(DataType::Real, DataType::Real);
  assert_eq!(DataType::Text, DataType::Text);
  assert_eq!(DataType::Blob, DataType::Blob);
}

#[test]
fn test_data_type_from_sql_type() {
  assert_eq!(DataType::from_sql_type("INTEGER"), Some(DataType::Integer));
  assert_eq!(DataType::from_sql_type("INT"), Some(DataType::Integer));
  assert_eq!(DataType::from_sql_type("REAL"), Some(DataType::Real));
  assert_eq!(DataType::from_sql_type("FLOAT"), Some(DataType::Real));
  assert_eq!(DataType::from_sql_type("DOUBLE"), Some(DataType::Real));
  assert_eq!(DataType::from_sql_type("TEXT"), Some(DataType::Text));
  assert_eq!(DataType::from_sql_type("VARCHAR"), Some(DataType::Text));
  assert_eq!(DataType::from_sql_type("CHAR"), Some(DataType::Text));
  assert_eq!(DataType::from_sql_type("STRING"), Some(DataType::Text));
  assert_eq!(DataType::from_sql_type("BLOB"), Some(DataType::Blob));
  assert_eq!(DataType::from_sql_type("BINARY"), Some(DataType::Blob));
  assert_eq!(DataType::from_sql_type("UNKNOWN"), None);
}

#[test]
fn test_data_type_matches_value() {
  use std::borrow::Cow;

  assert!(DataType::Integer.matches(&Value::Integer(1)));
  assert!(DataType::Real.matches(&Value::Real(1.0)));

  // NULL matches any
  assert!(DataType::Text.matches(&Value::Null));

  assert!(DataType::Text.matches(&Value::Text(Cow::Borrowed("x"))));
  assert!(DataType::Blob.matches(&Value::Blob(Cow::Borrowed(b"x"))));
}

#[test]
fn test_data_type_to_sql_type() {
  assert_eq!(DataType::Integer.to_sql_type(), "INTEGER");
  assert_eq!(DataType::Real.to_sql_type(), "REAL");
  assert_eq!(DataType::Text.to_sql_type(), "TEXT");
  assert_eq!(DataType::Blob.to_sql_type(), "BLOB");
}

// ===============================================
// Value 测试
// ===============================================

#[test]
fn test_value_null() {
  let value = Value::Null;
  assert_eq!(value.data_type(), DataType::Integer);
  assert_eq!(value.as_integer(), None);
  assert_eq!(value.as_real(), None);
  assert_eq!(value.as_text(), None);
  assert_eq!(value.as_blob(), None);
}

#[test]
fn test_value_integer() {
  let value = Value::Integer(123);
  assert_eq!(value.data_type(), DataType::Integer);
  assert_eq!(value.as_integer(), Some(123));
  assert_eq!(value.as_real(), None);
  assert_eq!(value.as_text(), None);
  assert_eq!(value.as_blob(), None);
}

#[test]
fn test_value_real() {
  let value = Value::Real(3.14);
  assert_eq!(value.data_type(), DataType::Real);
  assert_eq!(value.as_integer(), None);
  assert_eq!(value.as_real(), Some(3.14));
  assert_eq!(value.as_text(), None);
  assert_eq!(value.as_blob(), None);
}

#[test]
fn test_value_text_borrowed() {
  let s = "hello";
  let value = Value::Text(Cow::Borrowed(s));
  assert_eq!(value.data_type(), DataType::Text);
  assert_eq!(value.as_text(), Some("hello"));
  assert_eq!(value.as_integer(), None);
}

#[test]
fn test_value_text_owned() {
  let value = Value::Text(Cow::Owned("world".to_string()));
  assert_eq!(value.data_type(), DataType::Text);
  assert_eq!(value.as_text(), Some("world"));
}

#[test]
fn test_value_blob_borrowed() {
  let data = b"hello";
  let value = Value::Blob(Cow::Borrowed(data));
  assert_eq!(value.data_type(), DataType::Blob);
  assert_eq!(value.as_blob(), Some(b"hello" as &[u8]));
  assert_eq!(value.as_integer(), None);
}

#[test]
fn test_value_blob_owned() {
  let value = Value::Blob(Cow::Owned(b"world".to_vec()));
  assert_eq!(value.data_type(), DataType::Blob);
  assert_eq!(value.as_blob(), Some(b"world" as &[u8]));
}

#[test]
fn test_value_into_owned() {
  let s = "hello";
  let value = Value::Text(Cow::Borrowed(s));
  let owned: Value<'static> = value.into_owned();
  assert_eq!(owned.as_text(), Some("hello"));

  let data = b"world";
  let value = Value::Blob(Cow::Borrowed(data));
  let owned: Value<'static> = value.into_owned();
  assert_eq!(owned.as_blob(), Some(b"world" as &[u8]));
}

#[test]
fn test_value_sql_compare() {
  // 同类型比较
  assert_eq!(
    Value::Integer(1).sql_compare(&Value::Integer(2)),
    Some(Ordering::Less)
  );
  assert_eq!(
    Value::Integer(2).sql_compare(&Value::Integer(1)),
    Some(Ordering::Greater)
  );
  assert_eq!(
    Value::Integer(1).sql_compare(&Value::Integer(1)),
    Some(Ordering::Equal)
  );

  // Real 比较
  assert_eq!(
    Value::Real(1.0).sql_compare(&Value::Real(2.0)),
    Some(Ordering::Less)
  );

  // Text 比较
  assert_eq!(
    Value::Text(Cow::Borrowed("a")).sql_compare(&Value::Text(Cow::Borrowed("b"))),
    Some(Ordering::Less)
  );

  // NULL 比较
  assert_eq!(Value::Null.sql_compare(&Value::Integer(1)), None);
  assert_eq!(Value::Integer(1).sql_compare(&Value::Null), None);
  assert_eq!(Value::Null.sql_compare(&Value::Null), None);

  // 不同类型比较
  assert_eq!(Value::Integer(1).sql_compare(&Value::Real(1.0)), None);
}

// ===============================================
// ColumnConstraints 测试
// ===============================================

#[test]
fn test_column_constraints_defaults() {
  let constraints = ColumnConstraints::default();
  assert_eq!(constraints.not_null, false);
  assert_eq!(constraints.unique, false);
  assert_eq!(constraints.primary_key, false);
  assert_eq!(constraints.autoincrement, false);
}

#[test]
fn test_column_constraints_construction() {
  let constraints =
    ColumnConstraints { not_null: true, unique: true, primary_key: true, autoincrement: true };
  assert_eq!(constraints.not_null, true);
  assert_eq!(constraints.unique, true);
  assert_eq!(constraints.primary_key, true);
  assert_eq!(constraints.autoincrement, true);
}

// ===============================================
// Column 测试
// ===============================================

#[test]
fn test_column_new() {
  let column = Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer);
  assert_eq!(column.id, ColumnId::new(1));
  assert_eq!(column.name, "id");
  assert_eq!(column.data_type, DataType::Integer);
  assert_eq!(column.constraints.not_null, false);
  assert_eq!(column.default_value, None);
}

#[test]
fn test_column_with_constraints() {
  let constraints = ColumnConstraints { not_null: true, primary_key: true, ..Default::default() };
  let column = Column::with_constraints(
    ColumnId::new(1),
    "id".to_string(),
    DataType::Integer,
    constraints,
  );
  assert_eq!(column.constraints.not_null, true);
  assert_eq!(column.constraints.primary_key, true);
}

#[test]
fn test_column_validate_value_success() {
  let column = Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer);
  assert!(column.validate_value(&Value::Integer(123)).is_ok());
  assert!(column.validate_value(&Value::Null).is_ok());
}

#[test]
fn test_column_validate_value_not_null() {
  let mut column = Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer);
  column.constraints.not_null = true;

  assert!(column.validate_value(&Value::Integer(123)).is_ok());
  assert!(column.validate_value(&Value::Null).is_err());

  if let Err(DomainError::NotNullViolation { name }) = column.validate_value(&Value::Null) {
    assert_eq!(name, "id");
  } else {
    panic!("Expected NotNullViolation error");
  }
}

#[test]
fn test_column_validate_value_type_mismatch() {
  let column = Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer);
  assert!(column.validate_value(&Value::Real(3.14)).is_err());

  if let Err(DomainError::TypeMismatch { name, expected, got }) =
    column.validate_value(&Value::Real(3.14))
  {
    assert_eq!(name, "id");
    assert_eq!(expected, "INTEGER");
    assert_eq!(got, "REAL");
  } else {
    panic!("Expected TypeMismatch error");
  }
}

// ===============================================
// Table 测试
// ===============================================

#[test]
fn test_table_new() {
  let columns = vec![
    Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer),
    Column::new(ColumnId::new(2), "name".to_string(), DataType::Text),
  ];
  let table = Table::new(
    TableId::new(1),
    "users".to_string(),
    columns.clone(),
    Some(ColumnId::new(1)),
    PageId::new(1),
  );
  assert_eq!(table.id, TableId::new(1));
  assert_eq!(table.name, "users");
  assert_eq!(table.columns.len(), 2);
  assert_eq!(table.primary_key, Some(ColumnId::new(1)));
  assert_eq!(table.root_page, PageId::new(1));
}

#[test]
fn test_table_get_column() {
  let columns = vec![
    Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer),
    Column::new(ColumnId::new(2), "name".to_string(), DataType::Text),
  ];
  let table = Table::new(
    TableId::new(1),
    "users".to_string(),
    columns.clone(),
    Some(ColumnId::new(1)),
    PageId::new(1),
  );

  assert!(table.get_column("id").is_some());
  assert!(table.get_column("name").is_some());
  assert!(table.get_column("nonexistent").is_none());

  let id_column = table.get_column("id").unwrap();
  assert_eq!(id_column.name, "id");
  assert_eq!(id_column.data_type, DataType::Integer);
}

#[test]
fn test_table_primary_key_column() {
  let columns = vec![
    Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer),
    Column::new(ColumnId::new(2), "name".to_string(), DataType::Text),
  ];
  let table = Table::new(
    TableId::new(1),
    "users".to_string(),
    columns.clone(),
    Some(ColumnId::new(1)),
    PageId::new(1),
  );

  let pk_column = table.primary_key_column();
  assert!(pk_column.is_some());
  assert_eq!(pk_column.unwrap().name, "id");

  let table_no_pk = Table::new(
    TableId::new(2),
    "users".to_string(),
    columns.clone(),
    None,
    PageId::new(1),
  );
  assert!(table_no_pk.primary_key_column().is_none());
}

// ===============================================
// Row<'r> 测试
// ===============================================

#[test]
fn test_row_new() {
  let values = vec![Value::Integer(1), Value::Text(Cow::Borrowed("Alice"))];
  let row = Row::new(RowId::new(1), values);
  assert_eq!(row.row_id, RowId::new(1));
  assert_eq!(row.values.len(), 2);
}

#[test]
fn test_row_get() {
  let values = vec![Value::Integer(1), Value::Text(Cow::Borrowed("Alice"))];
  let row = Row::new(RowId::new(1), values);

  assert_eq!(row.get(0), Some(&Value::Integer(1)));
  assert_eq!(row.get(1), Some(&Value::Text(Cow::Borrowed("Alice"))));
  assert_eq!(row.get(2), None);
}

#[test]
fn test_row_get_by_name() {
  let columns = vec![
    Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer),
    Column::new(ColumnId::new(2), "name".to_string(), DataType::Text),
  ];
  let table = Table::new(
    TableId::new(1),
    "users".to_string(),
    columns,
    None,
    PageId::new(1),
  );

  let values = vec![Value::Integer(1), Value::Text(Cow::Borrowed("Alice"))];
  let row = Row::new(RowId::new(1), values);

  assert_eq!(row.get_by_name("id", &table), Some(&Value::Integer(1)));
  assert_eq!(
    row.get_by_name("name", &table),
    Some(&Value::Text(Cow::Borrowed("Alice")))
  );
  assert_eq!(row.get_by_name("nonexistent", &table), None);
}

#[test]
fn test_row_into_owned() {
  let s = "hello";
  let values = vec![Value::Text(Cow::Borrowed(s))];
  let row = Row::new(RowId::new(1), values);
  let owned: Row<'static> = row.into_owned();
  assert_eq!(owned.get(0).unwrap().as_text(), Some("hello"));
}

// ===============================================
// Database 测试
// ===============================================

#[test]
fn test_database_new() {
  let db = Database::new(Path::new("/tmp/test.db"));
  assert_eq!(db.path, Path::new("/tmp/test.db"));
  assert_eq!(db.tables.len(), 0);
  assert_eq!(db.indexes.len(), 0);
  assert_eq!(db.schema_version, 0);
}

#[test]
fn test_database_add_table() {
  let mut db = Database::new(Path::new("/tmp/test.db"));

  let columns = vec![Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer)];
  let table = Table::new(
    TableId::new(1),
    "users".to_string(),
    columns,
    None,
    PageId::new(1),
  );

  let table_id = db.add_table(table).unwrap();
  assert_eq!(table_id, TableId::new(1));
  assert_eq!(db.tables.len(), 1);
  assert_eq!(db.schema_version, 1);
}

#[test]
fn test_database_add_table_duplicate_name() {
  let mut db = Database::new(Path::new("/tmp/test.db"));

  let columns = vec![Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer)];
  let table1 = Table::new(
    TableId::new(1),
    "users".to_string(),
    columns.clone(),
    None,
    PageId::new(1),
  );
  let table2 = Table::new(
    TableId::new(2),
    "users".to_string(),
    columns,
    None,
    PageId::new(2),
  );

  assert!(db.add_table(table1).is_ok());
  assert!(db.add_table(table2.clone()).is_err());

  if let Err(DomainError::TableAlreadyExists { name }) = db.add_table(table2) {
    assert_eq!(name, "users");
  } else {
    panic!("Expected TableAlreadyExists error");
  }
}

#[test]
fn test_database_add_table_no_columns() {
  let mut db = Database::new(Path::new("/tmp/test.db"));

  let table = Table::new(
    TableId::new(1),
    "users".to_string(),
    vec![],
    None,
    PageId::new(1),
  );

  assert!(db.add_table(table.clone()).is_err());

  if let Err(DomainError::TableMusthHaveColumns) = db.add_table(table) {
    // do nothing
  } else {
    panic!("Expected TableMusthHaveColumns error");
  }
}

#[test]
fn test_database_drop_table() {
  let mut db = Database::new(Path::new("/tmp/test.db"));

  let columns = vec![Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer)];
  let table = Table::new(
    TableId::new(1),
    "users".to_string(),
    columns,
    None,
    PageId::new(1),
  );

  db.add_table(table).unwrap();
  assert_eq!(db.tables.len(), 1);

  db.drop_table(TableId::new(1)).unwrap();
  assert_eq!(db.tables.len(), 0);
  assert_eq!(db.schema_version, 2);
}

#[test]
fn test_database_drop_table_not_found() {
  let mut db = Database::new(Path::new("/tmp/test.db"));

  assert!(db.drop_table(TableId::new(1)).is_err());

  if let Err(DomainError::TableNotFound { table_id }) = db.drop_table(TableId::new(1)) {
    assert_eq!(table_id, TableId::new(1));
  } else {
    panic!("Expected TableNotFound error");
  }
}

#[test]
fn test_database_get_table() {
  let mut db = Database::new(Path::new("/tmp/test.db"));

  let columns = vec![Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer)];
  let table = Table::new(
    TableId::new(1),
    "users".to_string(),
    columns,
    None,
    PageId::new(1),
  );

  db.add_table(table).unwrap();

  let retrieved = db.get_table(TableId::new(1));
  assert!(retrieved.is_some());
  assert_eq!(retrieved.unwrap().name, "users");

  assert!(db.get_table(TableId::new(2)).is_none());
}

#[test]
fn test_database_get_table_by_name() {
  let mut db = Database::new(Path::new("/tmp/test.db"));

  let columns = vec![Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer)];
  let table = Table::new(
    TableId::new(1),
    "users".to_string(),
    columns,
    None,
    PageId::new(1),
  );

  db.add_table(table).unwrap();

  let retrieved = db.get_table_by_name("users");
  assert!(retrieved.is_some());
  assert_eq!(retrieved.unwrap().id, TableId::new(1));

  assert!(db.get_table_by_name("nonexistent").is_none());
}

#[test]
fn test_database_add_index() {
  let mut db = Database::new(Path::new("/tmp/test.db"));

  let columns = vec![Column::new(ColumnId::new(1), "id".to_string(), DataType::Integer)];
  let table = Table::new(
    TableId::new(1),
    "users".to_string(),
    columns,
    None,
    PageId::new(1),
  );
  db.add_table(table).unwrap();

  let index_id = db.add_index(IndexId::new(1), TableId::new(1)).unwrap();
  assert_eq!(index_id, IndexId::new(1));
  assert_eq!(db.indexes.len(), 1);
  assert_eq!(db.schema_version, 2);
}

// ===============================================
// DomainError 测试
// ===============================================

#[test]
fn test_domain_error_table_already_exists() {
  let error = DomainError::TableAlreadyExists { name: "users".to_string() };
  assert_eq!(error.to_string(), "Table 'users' already exists");
}

#[test]
fn test_domain_error_table_not_found() {
  let error = DomainError::TableNotFound { table_id: TableId::new(1) };
  assert!(error.to_string().contains("does not exist"));
}

#[test]
fn test_domain_error_not_null_violation() {
  let error = DomainError::NotNullViolation { name: "id".to_string() };
  assert_eq!(error.to_string(), "Column 'id' does not allow NULL values");
}

#[test]
fn test_domain_error_type_mismatch() {
  let error = DomainError::TypeMismatch {
    name: "id".to_string(),
    expected: "INTEGER".to_string(),
    got: "REAL".to_string(),
  };
  assert!(error.to_string().contains("type does not match"));
}
