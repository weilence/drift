use crate::data_type::DataType;
use crate::dialect::Dialect;
use std::process::Command;

pub trait MigrationStep {
    fn generate_sql(&self, table_name: &str, dialect: &dyn Dialect) -> String;
}

#[derive(Debug)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

#[derive(Debug)]
pub struct AddColumn {
    pub column: Column,
}

#[derive(Debug)]
pub struct DropColumn {
    pub name: String,
}

#[derive(Debug)]
pub struct RenameColumn {
    pub old_name: String,
    pub new_name: String,
}

#[derive(Debug)]
pub struct ColumnOptions {
    pub nullable: Option<bool>,
    pub default: Option<String>,
    pub unique: Option<bool>,
}

impl Default for ColumnOptions {
    fn default() -> Self {
        Self {
            nullable: None,
            default: None,
            unique: None,
        }
    }
}

#[derive(Debug)]
pub struct ChangeColumnType {
    pub column_name: String,
    pub new_type: DataType,
    pub options: ColumnOptions,
}

#[derive(Debug)]
pub enum UpdateValue {
    Fixed(String),
    Column(String),
}

#[derive(Debug)]
pub enum Operator {
    Eq,
    Ne,
    Gt,
    Lt,
    Gte,
    Lte,
    Like,
    In,
}

impl Operator {
    pub fn as_str(&self) -> &'static str {
        match self {
            Operator::Eq => "=",
            Operator::Ne => "!=",
            Operator::Gt => ">",
            Operator::Lt => "<",
            Operator::Gte => ">=",
            Operator::Lte => "<=",
            Operator::Like => "LIKE",
            Operator::In => "IN",
        }
    }
}

#[derive(Debug)]
pub struct WhereCondition {
    pub column: String,
    pub operator: Operator,
    pub value: UpdateValue,
}

#[derive(Debug)]
pub struct UpdateColumnData {
    pub column_name: String,
    pub value: UpdateValue,
    pub conditions: Vec<WhereCondition>,
}

#[derive(Debug)]
pub struct ExternalProcessColumnData {
    pub column_name: String,
    pub id_column: String,
    pub python_script: String,
}

impl MigrationStep for AddColumn {
    fn generate_sql(&self, table_name: &str, dialect: &dyn Dialect) -> String {
        dialect.add_column(
            table_name,
            &self.column.name,
            &self.column.data_type,
            self.column.nullable,
        )
    }
}

impl MigrationStep for DropColumn {
    fn generate_sql(&self, table_name: &str, dialect: &dyn Dialect) -> String {
        dialect.drop_column(table_name, &self.name)
    }
}

impl MigrationStep for RenameColumn {
    fn generate_sql(&self, table_name: &str, dialect: &dyn Dialect) -> String {
        dialect.rename_column(table_name, &self.old_name, &self.new_name)
    }
}

impl MigrationStep for ChangeColumnType {
    fn generate_sql(&self, table_name: &str, dialect: &dyn Dialect) -> String {
        dialect.change_column_type(
            table_name,
            &self.column_name,
            &self.new_type,
            &self.options,
        )
    }
}

impl MigrationStep for UpdateColumnData {
    fn generate_sql(&self, table_name: &str, dialect: &dyn Dialect) -> String {
        dialect.update_column_data(table_name, &self.column_name, &self.value, &self.conditions)
    }
}

impl MigrationStep for ExternalProcessColumnData {
    fn generate_sql(&self, table_name: &str, dialect: &dyn Dialect) -> String {
        // 1. Generate SELECT query with ID
        let select_sql = dialect.select_column_data(table_name, &self.id_column, &self.column_name);

        // 2. Execute Python script with the data
        let output = Command::new("python")
            .arg(&self.python_script)
            .arg(&select_sql)
            .output()
            .expect("Failed to execute Python script");

        // 3. Process Python script output
        // Expected format from Python: "id1:value1;id2:value2;..."
        let updates: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|pair| {
                let mut parts = pair.split(':');
                let id = parts.next().unwrap();
                let value = parts.next().unwrap();
                dialect.update_column_data_by_id(
                    table_name,
                    &self.id_column,
                    &self.column_name,
                    id,
                    value,
                )
            })
            .collect();

        updates.join("\n")
    }
}

pub struct Migration {
    table_name: String,
    operations: Vec<Box<dyn MigrationStep>>,
    dialect: Box<dyn Dialect>,
}

impl Migration {
    pub fn new(table_name: &str, dialect: Box<dyn Dialect>) -> Self {
        Migration {
            table_name: table_name.to_string(),
            operations: Vec::new(),
            dialect,
        }
    }

    pub fn add_operation<T: MigrationStep + 'static>(&mut self, operation: T) {
        self.operations.push(Box::new(operation));
    }

    pub fn generate_sql(&self) -> Vec<String> {
        self.operations
            .iter()
            .map(|op| op.generate_sql(&self.table_name, self.dialect.as_ref()))
            .collect()
    }
}
