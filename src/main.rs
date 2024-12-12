mod data_type;
mod dialect;
mod migration;

use data_type::DataType;
use dialect::{MySqlDialect, PostgresDialect};
use migration::{
    AddColumn, ChangeColumnType, Column, ColumnOptions, DropColumn, Migration, RenameColumn,
    UpdateColumnData,
};

pub fn main() {
    // PostgreSQL example
    let mut pg_migration = Migration::new("users", Box::new(PostgresDialect));

    // MySQL example
    let mut mysql_migration = Migration::new("users", Box::new(MySqlDialect));

    // Add operations to both migrations
    for migration in [&mut pg_migration, &mut mysql_migration] {
        migration.add_operation(AddColumn {
            column: Column {
                name: "email".to_string(),
                data_type: DataType::Text,
                nullable: false,
            },
        });

        migration.add_operation(RenameColumn {
            old_name: "phone".to_string(),
            new_name: "contact_number".to_string(),
        });

        migration.add_operation(ChangeColumnType {
            column_name: "age".to_string(),
            new_type: DataType::Integer,
            options: ColumnOptions::default(),
        });

        migration.add_operation(UpdateColumnData {
            column_name: "status".to_string(),
            value: migration::UpdateValue::Fixed("active".to_string()),
            conditions: vec![],
        });

        migration.add_operation(DropColumn {
            name: "temp".to_string(),
        });
    }

    // Print PostgreSQL statements
    println!("PostgreSQL:");
    for statement in pg_migration.generate_sql() {
        println!("  {}", statement);
    }

    // Print MySQL statements
    println!("\nMySQL:");
    for statement in mysql_migration.generate_sql() {
        println!("  {}", statement);
    }
}
