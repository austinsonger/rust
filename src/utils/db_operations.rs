use diesel::prelude::*;
use diesel::query_dsl::methods::FilterDsl;
use diesel::result::Error as DieselError;
use diesel::pg::Pg;
use std::fmt::Debug;
use tracing::{debug, error, warn};

use crate::{
    database::get_connection,
    errors::Error,
};

/// Generic function to find a record by ID
///
/// # Type Parameters
/// * `T` - The type of the model
/// * `ID` - The type of the ID
/// * `Table` - The type of the table
///
/// # Arguments
/// * `id` - The ID of the record to find
/// * `table` - The table to query
/// * `id_column` - The ID column to filter on
/// * `resource_name` - The name of the resource for error messages
///
/// # Returns
/// * `Result<T, Error>` - The record or an error
pub fn find_by_id<T, ID, Table>(
    id: ID,
    table: Table,
    id_column: impl ExpressionMethods<SqlType = diesel::sql_types::Integer>,
    resource_name: &str,
) -> Result<T, Error>
where
    T: Queryable<diesel::sql_types::Integer, Pg> + Debug,
    ID: AsExpression<diesel::sql_types::Integer> + Debug,
    Table: FilterDsl<diesel::expression::operators::Eq<id_column, ID>>,
    <Table as FilterDsl<diesel::expression::operators::Eq<id_column, ID>>>::Output: LoadQuery<diesel::PgConnection, T>,
{
    debug!("Finding {} with ID: {:?}", resource_name, id);

    let mut conn = get_connection()?;

    let result = table
        .filter(id_column.eq(id))
        .first::<T>(&mut conn)
        .map_err(|err| {
            match err {
                DieselError::NotFound => {
                    debug!("{} with ID {:?} not found", resource_name, id);
                    Error::not_found()
                },
                _ => {
                    error!(
                        error = %err,
                        id = ?id,
                        "Database error when finding {} by ID",
                        resource_name
                    );
                    Error::database_error(
                        format!("Error finding {} by ID: {}", resource_name, err),
                        Some(Box::new(err)),
                        None
                    )
                }
            }
        })?;

    debug!("Found {} with ID: {:?}", resource_name, id);
    Ok(result)
}

/// Generic function to insert a record
///
/// # Type Parameters
/// * `T` - The type of the model to return
/// * `U` - The type of the new record
/// * `Table` - The type of the table
///
/// # Arguments
/// * `new_record` - The new record to insert
/// * `table` - The table to insert into
/// * `resource_name` - The name of the resource for error messages
///
/// # Returns
/// * `Result<T, Error>` - The inserted record or an error
pub fn insert_record<T, U, Table>(
    new_record: &U,
    table: Table,
    resource_name: &str,
) -> Result<T, Error>
where
    T: Queryable<diesel::sql_types::Integer, Pg> + Debug,
    U: Insertable<Table> + Debug,
    <U as Insertable<Table>>::Values: diesel::expression::Expression,
    Table: diesel::Table,
    diesel::insert_into::IncompleteInsertStatement<Table, <U as Insertable<Table>>::Values>: diesel::query_builder::InsertStatement<Table, <U as Insertable<Table>>::Values>,
    diesel::query_builder::InsertStatement<Table, <U as Insertable<Table>>::Values>: LoadQuery<diesel::PgConnection, T>,
{
    debug!("Inserting new {}: {:?}", resource_name, new_record);

    let mut conn = get_connection()?;

    let result = diesel::insert_into(table)
        .values(new_record)
        .get_result::<T>(&mut conn)
        .map_err(|err| {
            error!(
                error = %err,
                "Database error when inserting {}",
                resource_name
            );
            Error::database_error(
                format!("Failed to create {}: {}", resource_name, err),
                Some(Box::new(err)),
                None
            )
        })?;

    debug!("Inserted new {}", resource_name);
    Ok(result)
}

/// Generic function to update a record
///
/// # Type Parameters
/// * `T` - The type of the model to return
/// * `U` - The type of the update record
/// * `ID` - The type of the ID
/// * `Table` - The type of the table
///
/// # Arguments
/// * `id` - The ID of the record to update
/// * `update_record` - The update record
/// * `table` - The table to update
/// * `id_column` - The ID column to filter on
/// * `resource_name` - The name of the resource for error messages
///
/// # Returns
/// * `Result<T, Error>` - The updated record or an error
pub fn update_record<T, U, ID, Table>(
    id: ID,
    update_record: &U,
    table: Table,
    id_column: impl ExpressionMethods<SqlType = diesel::sql_types::Integer>,
    resource_name: &str,
) -> Result<T, Error>
where
    T: Queryable<diesel::sql_types::Integer, Pg> + Debug,
    U: AsChangeset<Target = Table> + Debug,
    ID: AsExpression<diesel::sql_types::Integer> + Debug,
    Table: FilterDsl<diesel::expression::operators::Eq<id_column, ID>>,
    <Table as FilterDsl<diesel::expression::operators::Eq<id_column, ID>>>::Output: diesel::query_dsl::methods::FindDsl<ID>,
    diesel::query_builder::UpdateStatement<
        <Table as FilterDsl<diesel::expression::operators::Eq<id_column, ID>>>::Output,
        <U as AsChangeset>::Changeset,
    >: LoadQuery<diesel::PgConnection, T>,
{
    debug!("Updating {} with ID: {:?}, data: {:?}", resource_name, id, update_record);

    let mut conn = get_connection()?;

    let result = diesel::update(table.filter(id_column.eq(id)))
        .set(update_record)
        .get_result::<T>(&mut conn)
        .map_err(|err| {
            match err {
                DieselError::NotFound => {
                    debug!("{} with ID {:?} not found for update", resource_name, id);
                    Error::not_found()
                },
                _ => {
                    error!(
                        error = %err,
                        id = ?id,
                        "Database error when updating {}",
                        resource_name
                    );
                    Error::database_error(
                        format!("Failed to update {}: {}", resource_name, err),
                        Some(Box::new(err)),
                        None
                    )
                }
            }
        })?;

    debug!("Updated {} with ID: {:?}", resource_name, id);
    Ok(result)
}

/// Generic function to delete a record
///
/// # Type Parameters
/// * `ID` - The type of the ID
/// * `Table` - The type of the table
///
/// # Arguments
/// * `id` - The ID of the record to delete
/// * `table` - The table to delete from
/// * `id_column` - The ID column to filter on
/// * `resource_name` - The name of the resource for error messages
///
/// # Returns
/// * `Result<(), Error>` - Success or an error
pub fn delete_record<ID, Table>(
    id: ID,
    table: Table,
    id_column: impl ExpressionMethods<SqlType = diesel::sql_types::Integer>,
    resource_name: &str,
) -> Result<(), Error>
where
    ID: AsExpression<diesel::sql_types::Integer> + Debug,
    Table: FilterDsl<diesel::expression::operators::Eq<id_column, ID>>,
    <Table as FilterDsl<diesel::expression::operators::Eq<id_column, ID>>>::Output: diesel::query_dsl::methods::ExecuteDsl<diesel::PgConnection>,
{
    debug!("Deleting {} with ID: {:?}", resource_name, id);

    let mut conn = get_connection()?;

    let rows_affected = diesel::delete(table.filter(id_column.eq(id)))
        .execute(&mut conn)
        .map_err(|err| {
            error!(
                error = %err,
                id = ?id,
                "Database error when deleting {}",
                resource_name
            );
            Error::database_error(
                format!("Failed to delete {}: {}", resource_name, err),
                Some(Box::new(err)),
                None
            )
        })?;

    if rows_affected == 0 {
        debug!("{} with ID {:?} not found for deletion", resource_name, id);
        return Err(Error::not_found());
    }

    debug!("Deleted {} with ID: {:?}", resource_name, id);
    Ok(())
}
