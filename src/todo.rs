// We’re deriving the Serialize trait from the serde crate, and sqlx::FromRow which allows us to get a Todo from a SQLx query.

#[derive(Serialize, Clone, sqlx::FromRow)]
pub struct Todo {
    id: i64,
    body: String,
    completed: bool,
    created_at: NaiveDateTime, // We use the chrono::NaiveDateTime type to map SQL timestamps into Rust objects.
    updated_at: NaiveDateTime,
}

impl Todo {
    pub async fn list(dbpool: SqlitePool) -> Result<Vec<Todo>, Error> {
        // Selects all todos from the todos table.
        query_as("select * from todos")
            .fetch_all(&dbpool)
            .await
            .map_err(Into::into)
    }

    pub async fn read(dbpool: SqlitePool, id: i64) -> Result<Todo, Error> {
        // Selects one todo from the todos table with matching id field.
        query_as("select * from todos where id = ?")
            .bind(id)
            .fetch_one(&dbpool)
            .await
            .map_err(Into::into)
    }
}
