use crate::data_type::DataType;
use crate::migration::{ColumnOptions, UpdateValue, WhereCondition};

pub trait Dialect {
    fn add_column(&self, table: &str, column: &str, data_type: &DataType, nullable: bool)
        -> String;
    fn drop_column(&self, table: &str, column: &str) -> String;
    fn rename_column(&self, table: &str, old_name: &str, new_name: &str) -> String;
    fn change_column_type(
        &self,
        table: &str,
        column: &str,
        new_type: &DataType,
        options: &ColumnOptions,
    ) -> String;
    fn update_column_data(
        &self,
        table: &str,
        column: &str,
        value: &UpdateValue,
        conditions: &[WhereCondition],
    ) -> String;
    fn select_column_data(&self, table: &str, id_column: &str, value_column: &str) -> String;
    fn update_column_data_by_id(
        &self,
        table: &str,
        id_column: &str,
        update_column: &str,
        id_value: &str,
        new_value: &str,
    ) -> String;
}

pub struct PostgresDialect;
pub struct MySqlDialect;

impl Dialect for PostgresDialect {
    fn add_column(
        &self,
        table: &str,
        column: &str,
        data_type: &DataType,
        nullable: bool,
    ) -> String {
        format!(
            "ALTER TABLE {} ADD COLUMN {} {} {};",
            table,
            column,
            data_type,
            if nullable { "NULL" } else { "NOT NULL" }
        )
    }

    fn drop_column(&self, table: &str, column: &str) -> String {
        format!("ALTER TABLE {} DROP COLUMN {};", table, column)
    }

    fn rename_column(&self, table: &str, old_name: &str, new_name: &str) -> String {
        format!(
            "ALTER TABLE {} RENAME COLUMN {} TO {};",
            table, old_name, new_name
        )
    }

    fn change_column_type(
        &self,
        table: &str,
        column: &str,
        new_type: &DataType,
        options: &ColumnOptions,
    ) -> String {
        let mut statements = vec![format!(
            "ALTER TABLE {} ALTER COLUMN {} TYPE {}",
            table, column, new_type
        )];

        if let Some(nullable) = options.nullable {
            statements.push(format!(
                "ALTER TABLE {} ALTER COLUMN {} {} NULL",
                table,
                column,
                if nullable { "DROP NOT" } else { "SET NOT" }
            ));
        }

        if let Some(ref default_value) = options.default {
            statements.push(format!(
                "ALTER TABLE {} ALTER COLUMN {} SET DEFAULT {}",
                table, column, default_value
            ));
        }

        if let Some(unique) = options.unique {
            if unique {
                statements.push(format!(
                    "CREATE UNIQUE INDEX IF NOT EXISTS {}_{}_unique ON {} ({})",
                    table, column, table, column
                ));
            } else {
                statements.push(format!("DROP INDEX IF EXISTS {}_{}_unique", table, column));
            }
        }

        statements.join(";\n")
    }

    fn update_column_data(
        &self,
        table: &str,
        column: &str,
        value: &UpdateValue,
        conditions: &[WhereCondition],
    ) -> String {
        let value_sql = match value {
            UpdateValue::Fixed(val) => val.clone(),
            UpdateValue::Column(col) => col.clone(),
        };

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            let conditions: Vec<String> = conditions
                .iter()
                .map(|cond| {
                    let value = match &cond.value {
                        UpdateValue::Fixed(val) => val.clone(),
                        UpdateValue::Column(col) => col.clone(),
                    };
                    format!("{} {} {}", cond.column, cond.operator.as_str(), value)
                })
                .collect();
            format!(" WHERE {}", conditions.join(" AND "))
        };

        format!("UPDATE {} SET {} = {}{};", table, column, value_sql, where_clause)
    }

    fn select_column_data(&self, table: &str, id_column: &str, value_column: &str) -> String {
        format!("SELECT {}, {} FROM {};", id_column, value_column, table)
    }

    fn update_column_data_by_id(
        &self,
        table: &str,
        id_column: &str,
        update_column: &str,
        id_value: &str,
        new_value: &str,
    ) -> String {
        format!(
            "UPDATE {} SET {} = {} WHERE {} = {};",
            table, update_column, new_value, id_column, id_value
        )
    }
}

impl Dialect for MySqlDialect {
    fn add_column(
        &self,
        table: &str,
        column: &str,
        data_type: &DataType,
        nullable: bool,
    ) -> String {
        format!(
            "ALTER TABLE {} ADD COLUMN {} {} {};",
            table,
            column,
            data_type,
            if nullable { "NULL" } else { "NOT NULL" }
        )
    }

    fn drop_column(&self, table: &str, column: &str) -> String {
        format!("ALTER TABLE {} DROP COLUMN {};", table, column)
    }

    fn rename_column(&self, table: &str, old_name: &str, new_name: &str) -> String {
        format!(
            "ALTER TABLE {} CHANGE COLUMN {} {};",
            table, old_name, new_name
        )
    }

    fn change_column_type(
        &self,
        table: &str,
        column: &str,
        new_type: &DataType,
        options: &ColumnOptions,
    ) -> String {
        let mut definition = format!("{}", new_type);

        if let Some(nullable) = options.nullable {
            definition.push_str(if nullable { " NULL" } else { " NOT NULL" });
        }

        if let Some(ref default_value) = options.default {
            definition.push_str(&format!(" DEFAULT {}", default_value));
        }

        if let Some(true) = options.unique {
            definition.push_str(" UNIQUE");
        }

        format!(
            "ALTER TABLE {} MODIFY COLUMN {} {}",
            table, column, definition
        )
    }

    fn update_column_data(
        &self,
        table: &str,
        column: &str,
        value: &UpdateValue,
        conditions: &[WhereCondition],
    ) -> String {
        // MySQL实现与PostgreSQL相同
        let value_sql = match value {
            UpdateValue::Fixed(val) => val.clone(),
            UpdateValue::Column(col) => col.clone(),
        };

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            let conditions: Vec<String> = conditions
                .iter()
                .map(|cond| {
                    let value = match &cond.value {
                        UpdateValue::Fixed(val) => val.clone(),
                        UpdateValue::Column(col) => col.clone(),
                    };
                    format!("{} {} {}", cond.column, cond.operator.as_str(), value)
                })
                .collect();
            format!(" WHERE {}", conditions.join(" AND "))
        };

        format!("UPDATE {} SET {} = {}{};", table, column, value_sql, where_clause)
    }

    fn select_column_data(&self, table: &str, id_column: &str, value_column: &str) -> String {
        format!("SELECT {}, {} FROM {};", id_column, value_column, table)
    }

    fn update_column_data_by_id(
        &self,
        table: &str,
        id_column: &str,
        update_column: &str,
        id_value: &str,
        new_value: &str,
    ) -> String {
        format!(
            "UPDATE {} SET {} = {} WHERE {} = {};",
            table, update_column, new_value, id_column, id_value
        )
    }
}
