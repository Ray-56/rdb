# rdb 快速开始指南

**Feature**: rdb 嵌入式关系型数据库  
**Target Audience**: Rust 开发者  
**Estimated Time**: 5 分钟  
**Date**: 2025-12-10

## 概述

rdb 是一个纯 Rust 实现的嵌入式关系型数据库，提供类似 SQLite 的功能。本指南将帮助您在 5 分钟内上手 rdb。

---

## 安装

### 添加依赖

在您的 `Cargo.toml` 中添加 rdb：

```toml
[dependencies]
rdb-interface = "1.0"  # 公共 API crate
```

### 最低 Rust 版本

- Rust 1.75+ (stable channel, 2021 edition)

---

## Hello, rdb!

### 示例 1：创建数据库并插入数据

```rust
use rdb_interface::{Database, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 打开数据库（如果不存在则创建）
    let db = Database::open("hello.db")?;
    
    // 2. 创建表
    db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)")?;
    
    // 3. 插入数据
    db.execute("INSERT INTO users VALUES (1, 'Alice', 30)")?;
    db.execute("INSERT INTO users VALUES (2, 'Bob', 25)")?;
    db.execute("INSERT INTO users VALUES (3, 'Charlie', 35)")?;
    
    println!("✅ 数据插入成功！");
    
    Ok(())
}
```

运行：

```bash
cargo run
```

输出：

```
✅ 数据插入成功！
```

---

### 示例 2：查询数据

```rust
use rdb_interface::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("hello.db")?;
    
    // 查询所有用户
    let rows = db.query("SELECT * FROM users")?;
    
    println!("📋 用户列表:");
    for row in rows {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let age: i64 = row.get(2)?;
        println!("  ID: {}, Name: {}, Age: {}", id, name, age);
    }
    
    Ok(())
}
```

输出：

```
📋 用户列表:
  ID: 1, Name: Alice, Age: 30
  ID: 2, Name: Bob, Age: 25
  ID: 3, Name: Charlie, Age: 35
```

---

### 示例 3：带条件的查询

```rust
use rdb_interface::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("hello.db")?;
    
    // 查询年龄大于 25 的用户
    let rows = db.query("SELECT name, age FROM users WHERE age > 25")?;
    
    println!("🔍 年龄大于 25 的用户:");
    for row in rows {
        let name: String = row.get(0)?;
        let age: i64 = row.get(1)?;
        println!("  {} ({} 岁)", name, age);
    }
    
    Ok(())
}
```

输出：

```
🔍 年龄大于 25 的用户:
  Alice (30 岁)
  Charlie (35 岁)
```

---

### 示例 4：参数化查询（防止 SQL 注入）

```rust
use rdb_interface::{Database, Value};

fn find_user_by_name(db: &Database, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 使用 ? 占位符进行参数化查询
    let stmt = db.prepare("SELECT * FROM users WHERE name = ?")?;
    let rows = stmt.query(&[Value::Text(name.into())])?;
    
    for row in rows {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let age: i64 = row.get(2)?;
        println!("找到用户: ID={}, Name={}, Age={}", id, name, age);
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("hello.db")?;
    
    println!("🔎 查找用户 'Alice':");
    find_user_by_name(&db, "Alice")?;
    
    Ok(())
}
```

输出：

```
🔎 查找用户 'Alice':
找到用户: ID=1, Name=Alice, Age=30
```

---

## 核心功能

### 1. 事务支持

```rust
use rdb_interface::Database;

fn transfer_money(db: &Database, from: i64, to: i64, amount: i64) -> Result<(), Box<dyn std::error::Error>> {
    // 开始事务
    let tx = db.begin_transaction()?;
    
    // 扣款
    tx.execute(&format!("UPDATE accounts SET balance = balance - {} WHERE id = {}", amount, from))?;
    
    // 加款
    tx.execute(&format!("UPDATE accounts SET balance = balance + {} WHERE id = {}", amount, to))?;
    
    // 提交事务（原子性保证）
    tx.commit()?;
    
    println!("✅ 转账成功：从账户 {} 转出 {} 到账户 {}", from, amount, to);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("bank.db")?;
    
    // 创建表
    db.execute("CREATE TABLE accounts (id INTEGER PRIMARY KEY, balance INTEGER)")?;
    db.execute("INSERT INTO accounts VALUES (1, 1000)")?;
    db.execute("INSERT INTO accounts VALUES (2, 500)")?;
    
    // 转账
    transfer_money(&db, 1, 2, 200)?;
    
    // 查询余额
    let rows = db.query("SELECT * FROM accounts")?;
    for row in rows {
        let id: i64 = row.get(0)?;
        let balance: i64 = row.get(1)?;
        println!("账户 {} 余额: {}", id, balance);
    }
    
    Ok(())
}
```

输出：

```
✅ 转账成功：从账户 1 转出 200 到账户 2
账户 1 余额: 800
账户 2 余额: 700
```

---

### 2. 索引加速查询

```rust
use rdb_interface::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("products.db")?;
    
    // 创建表
    db.execute("CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT, price REAL)")?;
    
    // 创建索引（加速按名称查询）
    db.execute("CREATE INDEX idx_products_name ON products(name)")?;
    
    // 插入 10000 条数据
    println!("📊 插入 10000 条数据...");
    for i in 1..=10000 {
        db.execute(&format!("INSERT INTO products VALUES ({}, 'Product {}', {})", i, i, i * 10.5))?;
    }
    
    // 查询（使用索引）
    println!("🔍 查询 'Product 5000'...");
    let rows = db.query("SELECT * FROM products WHERE name = 'Product 5000'")?;
    for row in rows {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let price: f64 = row.get(2)?;
        println!("  找到: ID={}, Name={}, Price=${:.2}", id, name, price);
    }
    
    Ok(())
}
```

---

### 3. 聚合函数

```rust
use rdb_interface::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("employees.db")?;
    
    db.execute("CREATE TABLE employees (id INTEGER PRIMARY KEY, name TEXT, salary INTEGER, department TEXT)")?;
    db.execute("INSERT INTO employees VALUES (1, 'Alice', 80000, 'Engineering')")?;
    db.execute("INSERT INTO employees VALUES (2, 'Bob', 60000, 'Engineering')")?;
    db.execute("INSERT INTO employees VALUES (3, 'Charlie', 70000, 'Sales')")?;
    db.execute("INSERT INTO employees VALUES (4, 'David', 90000, 'Engineering')")?;
    
    // 使用聚合函数
    let rows = db.query("SELECT COUNT(*), AVG(salary), MAX(salary) FROM employees")?;
    for row in rows {
        let count: i64 = row.get(0)?;
        let avg_salary: f64 = row.get(1)?;
        let max_salary: i64 = row.get(2)?;
        println!("📊 统计结果:");
        println!("  员工总数: {}", count);
        println!("  平均工资: ${:.2}", avg_salary);
        println!("  最高工资: ${}", max_salary);
    }
    
    Ok(())
}
```

输出：

```
📊 统计结果:
  员工总数: 4
  平均工资: $75000.00
  最高工资: $90000
```

---

### 4. JOIN 查询

```rust
use rdb_interface::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open("store.db")?;
    
    // 创建表
    db.execute("CREATE TABLE customers (id INTEGER PRIMARY KEY, name TEXT)")?;
    db.execute("CREATE TABLE orders (id INTEGER PRIMARY KEY, customer_id INTEGER, product TEXT, amount REAL)")?;
    
    // 插入数据
    db.execute("INSERT INTO customers VALUES (1, 'Alice')")?;
    db.execute("INSERT INTO customers VALUES (2, 'Bob')")?;
    
    db.execute("INSERT INTO orders VALUES (1, 1, 'Laptop', 999.99)")?;
    db.execute("INSERT INTO orders VALUES (2, 1, 'Mouse', 29.99)")?;
    db.execute("INSERT INTO orders VALUES (3, 2, 'Keyboard', 79.99)")?;
    
    // JOIN 查询
    let rows = db.query(
        "SELECT customers.name, orders.product, orders.amount \
         FROM customers JOIN orders ON customers.id = orders.customer_id"
    )?;
    
    println!("🛒 订单列表:");
    for row in rows {
        let customer: String = row.get(0)?;
        let product: String = row.get(1)?;
        let amount: f64 = row.get(2)?;
        println!("  {} 购买了 {} (${:.2})", customer, product, amount);
    }
    
    Ok(())
}
```

输出：

```
🛒 订单列表:
  Alice 购买了 Laptop ($999.99)
  Alice 购买了 Mouse ($29.99)
  Bob 购买了 Keyboard ($79.99)
```

---

## 内存数据库

rdb 支持纯内存数据库（不写入磁盘）：

```rust
use rdb_interface::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用特殊路径 ":memory:" 创建内存数据库
    let db = Database::open(":memory:")?;
    
    db.execute("CREATE TABLE temp (id INTEGER PRIMARY KEY, value TEXT)")?;
    db.execute("INSERT INTO temp VALUES (1, 'In-Memory Data')")?;
    
    let rows = db.query("SELECT * FROM temp")?;
    for row in rows {
        let id: i64 = row.get(0)?;
        let value: String = row.get(1)?;
        println!("ID: {}, Value: {}", id, value);
    }
    
    // 数据库关闭后，数据消失
    
    Ok(())
}
```

---

## 高级配置

### 自定义打开选项

```rust
use rdb_interface::{Database, Options};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open_with_options(
        "my_database.db",
        Options::new()
            .cache_size_mb(16)              // 缓存大小 16MB
            .wal_auto_checkpoint(2000)      // WAL 达到 2000 页时自动 checkpoint
    )?;
    
    // ... 使用数据库
    
    Ok(())
}
```

### 只读模式

```rust
use rdb_interface::{Database, Options};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open_with_options(
        "readonly.db",
        Options::new().read_only(true)
    )?;
    
    // 只能查询，不能修改
    let rows = db.query("SELECT * FROM users")?;
    
    // 以下操作会失败：
    // db.execute("INSERT INTO users VALUES (1, 'Alice', 30)")?; // ❌ 错误
    
    Ok(())
}
```

---

## 错误处理

```rust
use rdb_interface::{Database, RdbError};

fn main() {
    let db = match Database::open("my_database.db") {
        Ok(db) => db,
        Err(RdbError::Io(e)) => {
            eprintln!("❌ IO 错误: {}", e);
            return;
        }
        Err(RdbError::Corruption(msg)) => {
            eprintln!("❌ 数据库损坏: {}", msg);
            return;
        }
        Err(e) => {
            eprintln!("❌ 未知错误: {}", e);
            return;
        }
    };
    
    // 处理 SQL 错误
    match db.execute("INSERT INTO users VALUES (1, 'Alice')") {
        Ok(affected) => println!("✅ 影响 {} 行", affected),
        Err(RdbError::SqlSyntax { message, position }) => {
            eprintln!("❌ SQL 语法错误（位置 {}）: {}", position, message);
        }
        Err(RdbError::ConstraintViolation(msg)) => {
            eprintln!("❌ 约束违反: {}", msg);
        }
        Err(e) => eprintln!("❌ 执行错误: {}", e),
    }
}
```

---

## 多线程使用

rdb 支持多线程并发读取：

```rust
use rdb_interface::Database;
use std::sync::Arc;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Arc::new(Database::open("shared.db")?);
    
    db.execute("CREATE TABLE counter (id INTEGER PRIMARY KEY, value INTEGER)")?;
    db.execute("INSERT INTO counter VALUES (1, 0)")?;
    
    // 启动 10 个读线程
    let mut handles = vec![];
    for i in 0..10 {
        let db_clone = Arc::clone(&db);
        let handle = thread::spawn(move || {
            let rows = db_clone.query("SELECT value FROM counter WHERE id = 1").unwrap();
            for row in rows {
                let value: i64 = row.get(0).unwrap();
                println!("线程 {} 读取到值: {}", i, value);
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
- ✅ 多个线程可以并发读取
- ⚠️ v1.0 仅支持单写多读（多个写事务会串行化）

---

## 性能提示

### 1. 使用事务批量插入

```rust
// ❌ 慢：每次插入都提交
for i in 1..=10000 {
    db.execute(&format!("INSERT INTO users VALUES ({}, 'User {}', 25)", i, i))?;
}

// ✅ 快：批量插入在一个事务中
let tx = db.begin_transaction()?;
for i in 1..=10000 {
    tx.execute(&format!("INSERT INTO users VALUES ({}, 'User {}', 25)", i, i))?;
}
tx.commit()?;
```

### 2. 创建索引加速查询

```rust
// 对频繁查询的列创建索引
db.execute("CREATE INDEX idx_users_name ON users(name)")?;

// 查询时自动使用索引
let rows = db.query("SELECT * FROM users WHERE name = 'Alice'")?;
```

### 3. 使用 LIMIT 限制结果集

```rust
// 只获取前 100 行
let rows = db.query("SELECT * FROM large_table LIMIT 100")?;
```

---

## 下一步

- 📖 阅读 [API 文档](https://docs.rs/rdb-interface)
- 🏗️ 查看 [架构设计](../plan.md)
- 🔧 探索 [高级特性](https://github.com/your-org/rdb/wiki)
- 💬 加入 [社区讨论](https://github.com/your-org/rdb/discussions)

---

## 常见问题

### Q: rdb 与 SQLite 有什么区别？

A: rdb 是纯 Rust 实现，无需 C FFI；采用 DDD 架构；从第一天起就为 MVCC 和集群化预留接口。性能目标是达到 SQLite 的 50-100%（v1.0）。

### Q: rdb 支持哪些 SQL 特性？

A: v1.0 支持：
- DDL: CREATE/DROP TABLE, CREATE/DROP INDEX
- DML: INSERT, UPDATE, DELETE, SELECT
- WHERE, ORDER BY, LIMIT, JOIN, 聚合函数
- 事务（BEGIN/COMMIT/ROLLBACK）

### Q: rdb 是否支持异步 IO？

A: v1.0 仅支持同步 API。v2.0 计划添加异步 API（`async fn`）。

### Q: 如何迁移现有 SQLite 数据库到 rdb？

A: v1.0 不提供自动迁移工具。您可以通过 SQL 导出/导入数据：

```bash
# 从 SQLite 导出
sqlite3 old.db ".dump" > dump.sql

# 导入到 rdb（通过 rdb CLI，计划在 Week 36 实现）
rdb new.db < dump.sql
```

---

**Quickstart Version**: 1.0  
**Last Updated**: 2025-12-10  
**Feedback**: 欢迎提交 Issue 和 PR！

