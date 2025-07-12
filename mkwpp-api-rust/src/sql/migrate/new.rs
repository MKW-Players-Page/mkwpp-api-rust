use futures::StreamExt;
use std::io::Write;

use super::TABLE_NAMES;

pub async fn export_data(pool: &sqlx::Pool<sqlx::Postgres>) {
    const PATH: &str = "./db/fixtures/new/export/";

    let transaction = pool.begin();

    let path = std::path::Path::new(PATH);
    if !path.exists() {
        std::fs::create_dir_all(path).expect("Could not create directory")
    }

    let mut transaction = transaction.await.expect("Couldn't begin transaction");

    for table_name in TABLE_NAMES {
        println!("| Exporting {table_name}");

        let mut copy_stream = transaction
            .copy_out_raw(&format!(
                r#"COPY {table_name} TO STDOUT WITH DELIMITER E'\t' HEADER;"#
            ))
            .await
            .expect("Couldn't copy out data raw");

        let file = std::fs::File::create(&format!("{PATH}{table_name}.sql"))
            .expect("Couldn't create file");
        let mut bufwriter = std::io::BufWriter::new(file);
        while let Some(Ok(data)) = copy_stream.next().await {
            bufwriter.write(&data).expect("Couldn't write data");
        }
    }
}

pub async fn import_data(pool: &sqlx::Pool<sqlx::Postgres>) {
    const PATH: &str = "./db/fixtures/new/import";

    let transaction = pool.begin();
    let path = std::path::Path::new(PATH);
    if !path.exists() {
        std::fs::create_dir_all(path).expect("Could not create directory")
    }

    let mut transaction = transaction.await.expect("Couldn't begin transaction");

    for table_name in TABLE_NAMES {
        println!("| Importing {table_name}");

        let file_read = match std::fs::read(format!("{}/{table_name}.sql", PATH)) {
            Ok(v) => v,
            Err(e) => {
                println!("Error reading file {table_name}.sql: {e}");
                continue;
            }
        };

        sqlx::query(&format!(
            r#"CREATE TEMPORARY TABLE temp_{table_name} AS TABLE {table_name} WITH NO DATA"#
        ))
        .execute(&mut *transaction)
        .await
        .expect("Couldn't create temporary table");

        let mut copy_stream = transaction
            .copy_in_raw(&format!(
                r#"COPY temp_{table_name} FROM STDIN WITH DELIMITER E'\t' HEADER;"#
            ))
            .await
            .expect("Couldn't copy in data raw");
        
        copy_stream
            .send(file_read.as_slice())
            .await
            .expect("Couldn't send data to stdin");

        copy_stream
            .finish()
            .await
            .expect("Couldn't finish copy stream");

        let column_names: Vec<String> = sqlx::query_scalar(&format!(
            r#"
                SELECT column_name::text
                FROM information_schema.columns
                WHERE table_schema = 'public' AND table_name = '{table_name}'
                ORDER BY ordinal_position
            "#
        ))
        .fetch_all(&mut *transaction)
        .await
        .expect("Couldn't get table column names");

        let rows = sqlx::query(&format!(
            r#"
                MERGE INTO {table_name} USING temp_{table_name}
                    ON {table_name}.id = temp_{table_name}.id
                WHEN MATCHED THEN
                    UPDATE SET ({}) = ROW(temp_{table_name}.*)
                WHEN NOT MATCHED THEN
                    INSERT VALUES (temp_{table_name}.*)
            "#,
            column_names.join(",")
        ))
        .execute(&mut *transaction)
        .await
        .expect("Couldn't overwrite data");

        println!("| Rows affected: {}", rows.rows_affected());
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction.");
}
