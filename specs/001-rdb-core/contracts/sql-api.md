# SQL 层 API 契约

**Feature**: rdb 嵌入式关系型数据库  
**Layer**: SQL Layer (`rdb-sql` crate)  
**Visibility**: `pub(crate)` - 内部 API  
**Date**: 2025-12-10

## 概述

本文档定义 rdb SQL 层的内部 API。此层负责 SQL 解析、查询计划、优化和执行。

---

## Parser API

### SqlParser 类型

**职责**：SQL 解析器（封装 sqlparser-rs）

```rust
/// SQL 解析器
/// 
/// 线程安全: Send + Sync
pub(crate) struct SqlParser {
    dialect: SQLiteDialect,
}

impl SqlParser {
    /// 创建新解析器
    pub(crate) fn new() -> Self;
    
    /// 解析 SQL 语句
    /// 
    /// # Parameters
    /// - `sql`: SQL 字符串
    /// 
    /// # Returns
    /// - `Ok(SqlStatement)`: 解析后的 AST
    /// - `Err(SqlError::Syntax)`: 语法错误
    pub(crate) fn parse(&self, sql: &str) -> Result<SqlStatement>;
    
    /// 解析多个 SQL 语句（分号分隔）
    pub(crate) fn parse_multi(&self, sql: &str) -> Result<Vec<SqlStatement>>;
}

/// SQL 语句 AST
#[derive(Debug, Clone)]
pub(crate) enum SqlStatement {
    /// CREATE TABLE
    CreateTable {
        table_name: String,
        columns: Vec<ColumnDef>,
        constraints: Vec<TableConstraint>,
    },
    
    /// DROP TABLE
    DropTable {
        table_name: String,
        if_exists: bool,
    },
    
    /// CREATE INDEX
    CreateIndex {
        index_name: String,
        table_name: String,
        columns: Vec<String>,
        is_unique: bool,
    },
    
    /// DROP INDEX
    DropIndex {
        index_name: String,
        if_exists: bool,
    },
    
    /// INSERT
    Insert {
        table_name: String,
        columns: Option<Vec<String>>,
        values: Vec<Vec<Expr>>,
    },
    
    /// SELECT
    Select {
        projection: Vec<SelectItem>,
        from: Vec<TableRef>,
        selection: Option<Expr>,
        group_by: Vec<Expr>,
        having: Option<Expr>,
        order_by: Vec<OrderByExpr>,
        limit: Option<u64>,
        offset: Option<u64>,
    },
    
    /// UPDATE
    Update {
        table_name: String,
        assignments: Vec<Assignment>,
        selection: Option<Expr>,
    },
    
    /// DELETE
    Delete {
        table_name: String,
        selection: Option<Expr>,
    },
    
    /// BEGIN TRANSACTION
    BeginTransaction,
    
    /// COMMIT
    Commit,
    
    /// ROLLBACK
    Rollback,
}

/// 列定义
#[derive(Debug, Clone)]
pub(crate) struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub not_null: bool,
    pub unique: bool,
    pub primary_key: bool,
    pub autoincrement: bool,
    pub default_value: Option<Expr>,
}

/// 表达式
#[derive(Debug, Clone)]
pub(crate) enum Expr {
    Literal(Literal),
    Identifier(String),
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expr>,
    },
    Function {
        name: String,
        args: Vec<Expr>,
    },
    Cast {
        expr: Box<Expr>,
        data_type: DataType,
    },
}

/// 字面量
#[derive(Debug, Clone)]
pub(crate) enum Literal {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

/// 二元运算符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BinaryOperator {
    // 算术
    Add, Sub, Mul, Div, Mod,
    // 比较
    Eq, NotEq, Lt, LtEq, Gt, GtEq,
    // 逻辑
    And, Or,
    // 其他
    Like, In,
}

/// 一元运算符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnaryOperator {
    Not,
    Minus,
    IsNull,
    IsNotNull,
}
```

---

## Planner API

### QueryPlanner 类型

**职责**：生成查询计划

```rust
/// 查询计划器
/// 
/// 线程安全: !Send + !Sync
pub(crate) struct QueryPlanner<'db> {
    database: &'db Database,
}

impl<'db> QueryPlanner<'db> {
    /// 创建新计划器
    pub(crate) fn new(database: &'db Database) -> Self;
    
    /// 生成逻辑计划
    pub(crate) fn create_logical_plan(&self, stmt: &SqlStatement) -> Result<LogicalPlan>;
    
    /// 生成物理计划
    pub(crate) fn create_physical_plan(&self, logical: &LogicalPlan) -> Result<PhysicalPlan>;
}

/// 逻辑计划（高层抽象）
#[derive(Debug, Clone)]
pub(crate) enum LogicalPlan {
    /// 表扫描
    TableScan {
        table_id: TableId,
        table_name: String,
        projection: Option<Vec<usize>>,
    },
    
    /// 索引扫描
    IndexScan {
        index_id: IndexId,
        table_id: TableId,
        search_key: Expr,
    },
    
    /// 过滤（WHERE）
    Filter {
        input: Box<LogicalPlan>,
        predicate: Expr,
    },
    
    /// 投影（SELECT 列）
    Project {
        input: Box<LogicalPlan>,
        expressions: Vec<Expr>,
    },
    
    /// 聚合（GROUP BY）
    Aggregate {
        input: Box<LogicalPlan>,
        group_by: Vec<Expr>,
        aggregates: Vec<AggregateExpr>,
    },
    
    /// JOIN
    Join {
        left: Box<LogicalPlan>,
        right: Box<LogicalPlan>,
        join_type: JoinType,
        on: Expr,
    },
    
    /// 排序（ORDER BY）
    Sort {
        input: Box<LogicalPlan>,
        order_by: Vec<OrderByExpr>,
    },
    
    /// 限制（LIMIT）
    Limit {
        input: Box<LogicalPlan>,
        limit: usize,
        offset: usize,
    },
}

/// 物理计划（执行层）
#[derive(Debug, Clone)]
pub(crate) enum PhysicalPlan {
    SeqScan {
        table_id: TableId,
        projection: Option<Vec<usize>>,
    },
    
    IndexScan {
        index_id: IndexId,
        table_id: TableId,
        search_key: Vec<Value<'static>>,
    },
    
    Filter {
        input: Box<PhysicalPlan>,
        predicate: Expr,
    },
    
    Project {
        input: Box<PhysicalPlan>,
        expressions: Vec<Expr>,
    },
    
    HashAggregate {
        input: Box<PhysicalPlan>,
        group_by: Vec<Expr>,
        aggregates: Vec<AggregateExpr>,
    },
    
    HashJoin {
        left: Box<PhysicalPlan>,
        right: Box<PhysicalPlan>,
        join_type: JoinType,
        on: Expr,
    },
    
    NestedLoopJoin {
        left: Box<PhysicalPlan>,
        right: Box<PhysicalPlan>,
        join_type: JoinType,
        on: Expr,
    },
    
    Sort {
        input: Box<PhysicalPlan>,
        order_by: Vec<OrderByExpr>,
    },
    
    Limit {
        input: Box<PhysicalPlan>,
        limit: usize,
        offset: usize,
    },
}

/// JOIN 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

/// 聚合表达式
#[derive(Debug, Clone)]
pub(crate) struct AggregateExpr {
    pub function: AggregateFunction,
    pub argument: Expr,
}

/// 聚合函数
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

/// ORDER BY 表达式
#[derive(Debug, Clone)]
pub(crate) struct OrderByExpr {
    pub expr: Expr,
    pub asc: bool,
    pub nulls_first: bool,
}
```

---

## Optimizer API

### QueryOptimizer 类型

**职责**：优化查询计划

```rust
/// 查询优化器
/// 
/// 线程安全: Send + Sync
pub(crate) struct QueryOptimizer;

impl QueryOptimizer {
    /// 优化逻辑计划
    /// 
    /// 优化规则：
    /// 1. 谓词下推（Predicate Pushdown）
    /// 2. 投影下推（Projection Pushdown）
    /// 3. 索引选择（Index Selection）
    /// 4. JOIN 顺序优化
    pub(crate) fn optimize(&self, plan: LogicalPlan) -> Result<LogicalPlan>;
    
    /// 选择索引
    /// 
    /// # Parameters
    /// - `table_id`: 表 ID
    /// - `predicate`: WHERE 条件
    /// 
    /// # Returns
    /// - `Some(index_id)`: 选中的索引
    /// - `None`: 无合适索引，使用全表扫描
    pub(crate) fn select_index(
        &self,
        table_id: TableId,
        predicate: &Expr,
        database: &Database,
    ) -> Option<IndexId>;
    
    /// 估算计划成本
    pub(crate) fn estimate_cost(&self, plan: &PhysicalPlan) -> f64;
}
```

---

## Executor API

### Executor Trait

**职责**：查询执行器（迭代器模式）

```rust
/// 查询执行器 Trait
/// 
/// 所有执行器都实现此 trait，提供流式查询结果
pub(crate) trait Executor: Iterator<Item = Result<Row>> {
    /// 执行计划说明（用于 EXPLAIN）
    fn explain(&self) -> String;
    
    /// 获取结果列的元数据
    fn columns(&self) -> &[ColumnMetadata];
}

/// 列元数据
#[derive(Debug, Clone)]
pub(crate) struct ColumnMetadata {
    pub name: String,
    pub data_type: DataType,
}
```

### 具体执行器

#### SeqScanExecutor

```rust
/// 全表扫描执行器
/// 
/// 线程安全: !Send + !Sync
pub(crate) struct SeqScanExecutor<'exec> {
    table_id: TableId,
    pager: &'exec Pager<'exec>,
    cursor: BTreeCursor<'exec, RowId, Row>,
    projection: Option<Vec<usize>>,
}

impl<'exec> SeqScanExecutor<'exec> {
    pub(crate) fn new(
        table_id: TableId,
        pager: &'exec Pager<'exec>,
        projection: Option<Vec<usize>>,
    ) -> Result<Self>;
}

impl<'exec> Iterator for SeqScanExecutor<'exec> {
    type Item = Result<Row>;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.cursor.next()
    }
}

impl<'exec> Executor for SeqScanExecutor<'exec> {
    fn explain(&self) -> String {
        format!("SeqScan(table_id={:?})", self.table_id)
    }
    
    fn columns(&self) -> &[ColumnMetadata] {
        // 返回表的列元数据
        todo!()
    }
}
```

#### IndexScanExecutor

```rust
/// 索引扫描执行器
pub(crate) struct IndexScanExecutor<'exec> {
    index_id: IndexId,
    table_id: TableId,
    pager: &'exec Pager<'exec>,
    cursor: BTreeCursor<'exec, IndexKey, RowId>,
}

impl<'exec> IndexScanExecutor<'exec> {
    pub(crate) fn new(
        index_id: IndexId,
        table_id: TableId,
        pager: &'exec Pager<'exec>,
        search_key: Vec<Value<'static>>,
    ) -> Result<Self>;
}

impl<'exec> Iterator for IndexScanExecutor<'exec> {
    type Item = Result<Row>;
    
    fn next(&mut self) -> Option<Self::Item> {
        // 1. 从索引 cursor 获取 RowId
        // 2. 从表 B+Tree 查找对应的 Row
        todo!()
    }
}

impl<'exec> Executor for IndexScanExecutor<'exec> {
    fn explain(&self) -> String {
        format!("IndexScan(index_id={:?})", self.index_id)
    }
    
    fn columns(&self) -> &[ColumnMetadata] {
        todo!()
    }
}
```

#### FilterExecutor

```rust
/// WHERE 过滤执行器
pub(crate) struct FilterExecutor<'exec> {
    input: Box<dyn Executor + 'exec>,
    predicate: Expr,
}

impl<'exec> FilterExecutor<'exec> {
    pub(crate) fn new(input: Box<dyn Executor + 'exec>, predicate: Expr) -> Self;
    
    /// 评估谓词
    fn evaluate_predicate(&self, row: &Row) -> Result<bool>;
}

impl<'exec> Iterator for FilterExecutor<'exec> {
    type Item = Result<Row>;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let row = self.input.next()?;
            match row {
                Ok(r) => {
                    match self.evaluate_predicate(&r) {
                        Ok(true) => return Some(Ok(r)),
                        Ok(false) => continue,
                        Err(e) => return Some(Err(e)),
                    }
                }
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

impl<'exec> Executor for FilterExecutor<'exec> {
    fn explain(&self) -> String {
        format!("Filter(predicate={:?}) -> {}", self.predicate, self.input.explain())
    }
    
    fn columns(&self) -> &[ColumnMetadata] {
        self.input.columns()
    }
}
```

#### ProjectExecutor

```rust
/// SELECT 列投影执行器
pub(crate) struct ProjectExecutor<'exec> {
    input: Box<dyn Executor + 'exec>,
    expressions: Vec<Expr>,
    output_columns: Vec<ColumnMetadata>,
}

impl<'exec> ProjectExecutor<'exec> {
    pub(crate) fn new(
        input: Box<dyn Executor + 'exec>,
        expressions: Vec<Expr>,
    ) -> Result<Self>;
    
    /// 评估表达式
    fn evaluate_expr(&self, expr: &Expr, row: &Row) -> Result<Value<'static>>;
}

impl<'exec> Iterator for ProjectExecutor<'exec> {
    type Item = Result<Row>;
    
    fn next(&mut self) -> Option<Self::Item> {
        let input_row = self.input.next()?;
        match input_row {
            Ok(r) => {
                let mut values = Vec::new();
                for expr in &self.expressions {
                    match self.evaluate_expr(expr, &r) {
                        Ok(v) => values.push(v),
                        Err(e) => return Some(Err(e)),
                    }
                }
                Some(Ok(Row::new(values)))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'exec> Executor for ProjectExecutor<'exec> {
    fn explain(&self) -> String {
        format!("Project(expressions={:?}) -> {}", self.expressions, self.input.explain())
    }
    
    fn columns(&self) -> &[ColumnMetadata] {
        &self.output_columns
    }
}
```

#### HashJoinExecutor

```rust
/// Hash Join 执行器
pub(crate) struct HashJoinExecutor<'exec> {
    left: Box<dyn Executor + 'exec>,
    right: Box<dyn Executor + 'exec>,
    join_type: JoinType,
    on: Expr,
    hash_table: HashMap<Vec<Value<'static>>, Vec<Row>>,
}

impl<'exec> HashJoinExecutor<'exec> {
    pub(crate) fn new(
        left: Box<dyn Executor + 'exec>,
        right: Box<dyn Executor + 'exec>,
        join_type: JoinType,
        on: Expr,
    ) -> Result<Self>;
    
    /// 构建 Hash 表（从右表）
    fn build_hash_table(&mut self) -> Result<()>;
    
    /// 探测 Hash 表（从左表）
    fn probe(&mut self, left_row: &Row) -> Result<Vec<Row>>;
}

impl<'exec> Iterator for HashJoinExecutor<'exec> {
    type Item = Result<Row>;
    
    fn next(&mut self) -> Option<Self::Item> {
        // 实现 Hash Join 算法
        todo!()
    }
}

impl<'exec> Executor for HashJoinExecutor<'exec> {
    fn explain(&self) -> String {
        format!(
            "HashJoin(type={:?}) -> Left: {}, Right: {}",
            self.join_type,
            self.left.explain(),
            self.right.explain()
        )
    }
    
    fn columns(&self) -> &[ColumnMetadata] {
        // 合并左右表列
        todo!()
    }
}
```

---

## Expression Evaluator

### ExprEvaluator 类型

**职责**：评估表达式

```rust
/// 表达式求值器
pub(crate) struct ExprEvaluator;

impl ExprEvaluator {
    /// 评估表达式
    /// 
    /// # Parameters
    /// - `expr`: 表达式
    /// - `row`: 当前行（提供列值）
    /// 
    /// # Returns
    /// - `Ok(Value)`: 表达式结果
    pub(crate) fn evaluate(&self, expr: &Expr, row: &Row) -> Result<Value<'static>>;
    
    /// 评估二元运算
    fn evaluate_binary_op(
        &self,
        op: BinaryOperator,
        left: &Value,
        right: &Value,
    ) -> Result<Value<'static>>;
    
    /// 评估一元运算
    fn evaluate_unary_op(&self, op: UnaryOperator, value: &Value) -> Result<Value<'static>>;
    
    /// 评估函数调用
    fn evaluate_function(&self, name: &str, args: &[Value]) -> Result<Value<'static>>;
}
```

---

## 错误类型

```rust
/// SQL 层错误
#[derive(Debug, thiserror::Error)]
pub(crate) enum SqlError {
    #[error("SQL syntax error at position {position}: {message}")]
    Syntax { message: String, position: usize },
    
    #[error("Table not found: {0}")]
    TableNotFound(String),
    
    #[error("Column not found: {0}")]
    ColumnNotFound(String),
    
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("Invalid expression: {0}")]
    InvalidExpression(String),
    
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}
```

---

**SQL API Version**: 1.0  
**Internal Use Only**: `pub(crate)`  
**Date**: 2025-12-10

