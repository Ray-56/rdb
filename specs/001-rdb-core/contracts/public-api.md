# 公共 Rust API 契约

**Feature**: rdb 嵌入式关系型数据库  
**Layer**: Interface Layer (`rdb-interface` crate)  
**Date**: 2025-12-10

## 概述

本文档定义 rdb 的公共 Rust API，这是唯一暴露给外部用户的接口。所有 API 必须：
- 类型安全（充分利用 Rust 类型系统）
- 内存安全（无 unsafe 泄漏到公共 API）
- 线程安全（明确标注 Send/Sync 要求）
- 错误友好（提供清晰的错误信息）

---

## 核心 API

### 1. Database 类型

**职责**：数据库实例，管理连接和全局状态

```rust
/// 数据库实例
/// 
/// 线程安全: Send + Sync (内部使用 Arc)
/// 
/// # Examples
/// 
/// ```
/// use rdb_interface::Database;
/// 
/// let db = Database::open("test.db")?;
/// db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")?;
/// ```
pub struct Database {
    // 内部实现（不暴露给用户）
    inner: Arc<DatabaseInner>,
}

impl Database {
    /// 打开或创建数据库
    /// 
    /// # Parameters
    /// - `path`: 数据库文件路径（如果不存在则创建）
    /// 
    /// # Returns
    /// - `Ok(Database)`: 成功打开数据库
    /// - `Err(RdbError)`: IO 错误或数据库损坏
    /// 
    /// # Examples
    /// 
    /// ```
    /// // 打开磁盘数据库
    /// let db = Database::open("my_database.db")?;
    /// 
    /// // 创建内存数据库
    /// let db = Database::open(":memory:")?;
    /// ```
    pub fn open(path: impl AsRef<Path>) -> Result<Self, RdbError>;
    
    /// 打开数据库（带选项）
    /// 
    /// # Parameters
    /// - `path`: 数据库文件路径
    /// - `options`: 打开选项（只读、缓存大小等）
    /// 
    /// # Examples
    /// 
    /// ```
    /// let db = Database::open_with_options(
    ///     "readonly.db",
    ///     Options::new().read_only(true).cache_size_mb(16)
    /// )?;
    /// ```
    pub fn open_with_options(
        path: impl AsRef<Path>,
        options: Options,
    ) -> Result<Self, RdbError>;
    
    /// 执行 SQL 语句（DDL 或 DML）
    /// 
    /// # Parameters
    /// - `sql`: SQL 语句字符串
    /// 
    /// # Returns
    /// - `Ok(usize)`: 影响的行数（INSERT/UPDATE/DELETE）或 0（DDL）
    /// - `Err(RdbError)`: SQL 语法错误或执行错误
    /// 
    /// # Examples
    /// 
    /// ```
    /// // CREATE TABLE
    /// db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")?;
    /// 
    /// // INSERT
    /// let affected = db.execute("INSERT INTO users VALUES (1, 'Alice')")?;
    /// assert_eq!(affected, 1);
    /// 
    /// // UPDATE
    /// let affected = db.execute("UPDATE users SET name = 'Bob' WHERE id = 1")?;
    /// ```
    pub fn execute(&self, sql: &str) -> Result<usize, RdbError>;
    
    /// 查询数据（返回迭代器）
    /// 
    /// # Parameters
    /// - `sql`: SELECT 查询语句
    /// 
    /// # Returns
    /// - `Ok(Rows)`: 结果行迭代器
    /// - `Err(RdbError)`: SQL 错误或执行错误
    /// 
    /// # Examples
    /// 
    /// ```
    /// let rows = db.query("SELECT * FROM users")?;
    /// for row in rows {
    ///     let id: i64 = row.get(0)?;
    ///     let name: String = row.get(1)?;
    ///     println!("id={}, name={}", id, name);
    /// }
    /// ```
    pub fn query(&self, sql: &str) -> Result<Rows, RdbError>;
    
    /// 准备参数化语句（防止 SQL 注入）
    /// 
    /// # Parameters
    /// - `sql`: 带占位符的 SQL 语句（使用 `?` 作为占位符）
    /// 
    /// # Returns
    /// - `Ok(Statement)`: 预编译的语句
    /// 
    /// # Examples
    /// 
    /// ```
    /// let stmt = db.prepare("SELECT * FROM users WHERE id = ?")?;
    /// let rows = stmt.query(&[Value::Integer(1)])?;
    /// ```
    pub fn prepare(&self, sql: &str) -> Result<Statement, RdbError>;
    
    /// 开始事务
    /// 
    /// # Examples
    /// 
    /// ```
    /// let tx = db.begin_transaction()?;
    /// tx.execute("INSERT INTO users VALUES (1, 'Alice')")?;
    /// tx.execute("INSERT INTO users VALUES (2, 'Bob')")?;
    /// tx.commit()?; // 提交所有更改
    /// ```
    pub fn begin_transaction(&self) -> Result<Transaction, RdbError>;
    
    /// 关闭数据库（释放资源）
    /// 
    /// 注意：数据库在 Drop 时自动关闭，此方法用于显式关闭并处理错误
    pub fn close(self) -> Result<(), RdbError>;
    
    /// 执行 Checkpoint（将 WAL 同步到主文件）
    /// 
    /// # Examples
    /// 
    /// ```
    /// db.checkpoint()?; // 同步 WAL 到磁盘
    /// ```
    pub fn checkpoint(&self) -> Result<(), RdbError>;
}
```

---

### 2. Transaction 类型

**职责**：事务上下文

```rust
/// 事务
/// 
/// 线程安全: !Send + !Sync (必须在创建线程使用)
/// 
/// 事务必须以 `commit()` 或 `rollback()` 结束，否则自动回滚
pub struct Transaction<'db> {
    inner: TransactionInner<'db>,
}

impl<'db> Transaction<'db> {
    /// 执行 SQL 语句（在事务上下文中）
    pub fn execute(&self, sql: &str) -> Result<usize, RdbError>;
    
    /// 查询数据（在事务上下文中）
    pub fn query(&self, sql: &str) -> Result<Rows<'db>, RdbError>;
    
    /// 准备语句（在事务上下文中）
    pub fn prepare(&self, sql: &str) -> Result<Statement<'db>, RdbError>;
    
    /// 提交事务
    /// 
    /// # Examples
    /// 
    /// ```
    /// let tx = db.begin_transaction()?;
    /// tx.execute("INSERT INTO users VALUES (1, 'Alice')")?;
    /// tx.commit()?; // 持久化所有更改
    /// ```
    pub fn commit(self) -> Result<(), RdbError>;
    
    /// 回滚事务
    /// 
    /// # Examples
    /// 
    /// ```
    /// let tx = db.begin_transaction()?;
    /// tx.execute("INSERT INTO users VALUES (1, 'Alice')")?;
    /// tx.rollback()?; // 撤销所有更改
    /// ```
    pub fn rollback(self) -> Result<(), RdbError>;
}

impl<'db> Drop for Transaction<'db> {
    /// 自动回滚（如果未显式提交）
    fn drop(&mut self) {
        // 如果事务未提交，自动回滚
    }
}
```

---

### 3. Statement 类型

**职责**：预编译的 SQL 语句（参数化查询）

```rust
/// 预编译语句
/// 
/// 线程安全: !Send + !Sync
pub struct Statement<'db> {
    inner: StatementInner<'db>,
}

impl<'db> Statement<'db> {
    /// 执行语句（绑定参数）
    /// 
    /// # Parameters
    /// - `params`: 参数值数组（按 `?` 占位符顺序）
    /// 
    /// # Examples
    /// 
    /// ```
    /// let stmt = db.prepare("INSERT INTO users VALUES (?, ?)")?;
    /// stmt.execute(&[
    ///     Value::Integer(1),
    ///     Value::Text("Alice".into()),
    /// ])?;
    /// ```
    pub fn execute(&self, params: &[Value]) -> Result<usize, RdbError>;
    
    /// 查询数据（绑定参数）
    /// 
    /// # Examples
    /// 
    /// ```
    /// let stmt = db.prepare("SELECT * FROM users WHERE id = ?")?;
    /// let rows = stmt.query(&[Value::Integer(1)])?;
    /// ```
    pub fn query(&self, params: &[Value]) -> Result<Rows<'db>, RdbError>;
    
    /// 清除绑定的参数
    pub fn reset(&mut self) -> Result<(), RdbError>;
}
```

---

### 4. Rows 迭代器

**职责**：查询结果集（流式迭代）

```rust
/// 查询结果迭代器
/// 
/// 线程安全: !Send + !Sync
/// 
/// # Examples
/// 
/// ```
/// let rows = db.query("SELECT * FROM users")?;
/// for row in rows {
///     let id: i64 = row.get(0)?;
///     let name: String = row.get(1)?;
/// }
/// ```
pub struct Rows<'db> {
    inner: RowsInner<'db>,
}

impl<'db> Iterator for Rows<'db> {
    type Item = Result<Row, RdbError>;
    
    fn next(&mut self) -> Option<Self::Item>;
}

impl<'db> Rows<'db> {
    /// 收集所有行到 Vec（注意内存占用）
    /// 
    /// # Examples
    /// 
    /// ```
    /// let rows = db.query("SELECT * FROM users")?;
    /// let all_rows: Vec<Row> = rows.collect::<Result<_, _>>()?;
    /// ```
    pub fn collect_vec(self) -> Result<Vec<Row>, RdbError>;
}
```

---

### 5. Row 类型

**职责**：单行数据

```rust
/// 单行数据
/// 
/// 线程安全: Send + Sync
#[derive(Debug, Clone)]
pub struct Row {
    values: Vec<Value<'static>>,
}

impl Row {
    /// 获取列值（按索引）
    /// 
    /// # Type Safety
    /// 
    /// 使用泛型自动转换类型：
    /// 
    /// ```
    /// let id: i64 = row.get(0)?;
    /// let name: String = row.get(1)?;
    /// let data: Vec<u8> = row.get(2)?;
    /// ```
    pub fn get<T: FromValue>(&self, index: usize) -> Result<T, RdbError>;
    
    /// 获取列值（按列名）
    /// 
    /// 注意：需要额外的列名元数据，性能较低
    pub fn get_by_name<T: FromValue>(&self, name: &str) -> Result<T, RdbError>;
    
    /// 获取列数
    pub fn len(&self) -> usize;
    
    /// 获取原始 Value
    pub fn get_value(&self, index: usize) -> Option<&Value>;
}
```

---

### 6. Value 类型

**职责**：数据库值（类型安全）

```rust
/// 数据库值
/// 
/// 线程安全: Send + Sync
#[derive(Debug, Clone, PartialEq)]
pub enum Value<'v> {
    Null,
    Integer(i64),
    Real(f64),
    Text(Cow<'v, str>),
    Blob(Cow<'v, [u8]>),
}

impl Value<'_> {
    /// 转换为拥有所有权的值
    pub fn into_owned(self) -> Value<'static>;
    
    /// 判断是否为 NULL
    pub fn is_null(&self) -> bool;
}

/// 从 Value 转换的 trait
pub trait FromValue: Sized {
    fn from_value(value: &Value) -> Result<Self, RdbError>;
}

/// 实现常见类型的转换
impl FromValue for i64 { /* ... */ }
impl FromValue for f64 { /* ... */ }
impl FromValue for String { /* ... */ }
impl FromValue for Vec<u8> { /* ... */ }
impl FromValue for bool { /* ... */ }
impl<T: FromValue> FromValue for Option<T> { /* ... */ }
```

---

### 7. Options 类型

**职责**：数据库打开选项

```rust
/// 数据库打开选项
/// 
/// 线程安全: Send + Sync
#[derive(Debug, Clone)]
pub struct Options {
    read_only: bool,
    cache_size_mb: usize,
    wal_auto_checkpoint: usize,
}

impl Options {
    /// 创建默认选项
    pub fn new() -> Self;
    
    /// 设置只读模式
    pub fn read_only(mut self, read_only: bool) -> Self;
    
    /// 设置缓存大小（MB）
    pub fn cache_size_mb(mut self, size: usize) -> Self;
    
    /// 设置 WAL 自动 checkpoint 阈值（页数）
    pub fn wal_auto_checkpoint(mut self, threshold: usize) -> Self;
}

impl Default for Options {
    fn default() -> Self {
        Self {
            read_only: false,
            cache_size_mb: 4,
            wal_auto_checkpoint: 1000,
        }
    }
}
```

---

### 8. RdbError 类型

**职责**：错误类型（用户友好）

```rust
/// rdb 错误类型
/// 
/// 线程安全: Send + Sync
#[derive(Debug, thiserror::Error)]
pub enum RdbError {
    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// SQL 语法错误
    #[error("SQL syntax error at position {position}: {message}")]
    SqlSyntax {
        message: String,
        position: usize,
    },
    
    /// SQL 执行错误
    #[error("SQL execution error: {0}")]
    SqlExecution(String),
    
    /// 约束违反
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    
    /// 数据库损坏
    #[error("Database corruption detected: {0}")]
    Corruption(String),
    
    /// 事务错误
    #[error("Transaction error: {0}")]
    Transaction(String),
    
    /// 类型转换错误
    #[error("Type conversion error: cannot convert {from} to {to}")]
    TypeConversion {
        from: String,
        to: String,
    },
    
    /// 列不存在
    #[error("Column not found: {0}")]
    ColumnNotFound(String),
    
    /// 表不存在
    #[error("Table not found: {0}")]
    TableNotFound(String),
}
```

---

## 使用示例

### 示例 1：基础 CRUD 操作

```rust
use rdb_interface::{Database, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 打开数据库
    let db = Database::open("my_app.db")?;
    
    // 创建表
    db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)")?;
    
    // 插入数据
    db.execute("INSERT INTO users VALUES (1, 'Alice', 30)")?;
    db.execute("INSERT INTO users VALUES (2, 'Bob', 25)")?;
    
    // 查询数据
    let rows = db.query("SELECT * FROM users WHERE age > 20")?;
    for row in rows {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let age: i64 = row.get(2)?;
        println!("User: id={}, name={}, age={}", id, name, age);
    }
    
    // 更新数据
    db.execute("UPDATE users SET age = 31 WHERE id = 1")?;
    
    // 删除数据
    db.execute("DELETE FROM users WHERE id = 2")?;
    
    Ok(())
}
```

### 示例 2：参数化查询（防止 SQL 注入）

```rust
use rdb_interface::{Database, Value};

fn find_user_by_name(db: &Database, name: &str) -> Result<Option<i64>, rdb_interface::RdbError> {
    let stmt = db.prepare("SELECT id FROM users WHERE name = ?")?;
    let mut rows = stmt.query(&[Value::Text(name.into())])?;
    
    if let Some(row) = rows.next() {
        let id: i64 = row?.get(0)?;
        Ok(Some(id))
    } else {
        Ok(None)
    }
}
```

### 示例 3：事务

```rust
use rdb_interface::Database;

fn transfer_money(db: &Database, from: i64, to: i64, amount: i64) -> Result<(), rdb_interface::RdbError> {
    let tx = db.begin_transaction()?;
    
    // 扣款
    tx.execute(&format!("UPDATE accounts SET balance = balance - {} WHERE id = {}", amount, from))?;
    
    // 加款
    tx.execute(&format!("UPDATE accounts SET balance = balance + {} WHERE id = {}", amount, to))?;
    
    // 提交事务
    tx.commit()?;
    
    Ok(())
}
```

### 示例 4：索引优化

```rust
use rdb_interface::Database;

fn setup_database(db: &Database) -> Result<(), rdb_interface::RdbError> {
    // 创建表
    db.execute("CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT, price REAL)")?;
    
    // 创建索引（加速按名称查询）
    db.execute("CREATE INDEX idx_products_name ON products(name)")?;
    
    // 插入数据
    for i in 1..10000 {
        db.execute(&format!("INSERT INTO products VALUES ({}, 'Product {}', {})", i, i, i * 10))?;
    }
    
    // 查询使用索引
    let rows = db.query("SELECT * FROM products WHERE name = 'Product 5000'")?;
    
    Ok(())
}
```

---

## 线程安全说明

### 多线程使用模式

```rust
use rdb_interface::Database;
use std::sync::Arc;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Arc::new(Database::open("shared.db")?);
    
    // 创建表
    db.execute("CREATE TABLE counter (id INTEGER PRIMARY KEY, value INTEGER)")?;
    db.execute("INSERT INTO counter VALUES (1, 0)")?;
    
    // 启动多个读线程
    let mut handles = vec![];
    for i in 0..10 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            // 每个线程可以并发读取
            let rows = db_clone.query("SELECT value FROM counter WHERE id = 1").unwrap();
            for row in rows {
                let value: i64 = row.get(0).unwrap();
                println!("Thread {}: value = {}", i, value);
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    Ok(())
}
```

**注意**：
- ✅ `Database` 是 `Send + Sync`，可以跨线程共享（通过 `Arc`）
- ✅ 多个线程可以并发读取
- ❌ `Transaction` 和 `Statement` 是 `!Send + !Sync`，必须在创建线程使用
- ⚠️ v1.0 仅支持单写多读，多个写事务会串行化

---

## API 稳定性保证

### v1.0 稳定 API（不会破坏性更改）

- `Database::open()`
- `Database::execute()`
- `Database::query()`
- `Database::begin_transaction()`
- `Transaction::commit()`
- `Transaction::rollback()`
- `Row::get()`
- `Value` 枚举的 5 个变体

### v1.x 可能添加（向后兼容）

- `Database::backup()` - 数据库备份
- `Database::analyze()` - 查询优化统计
- `Options::journal_mode()` - 切换日志模式
- `Value::DateTime` - 日期时间类型

### v2.0 可能破坏性更改

- MVCC 快照读 API
- 异步 API（`async fn`）

---

**API Contract Version**: 1.0  
**Stability**: Stable (v1.0+)  
**Date**: 2025-12-10

