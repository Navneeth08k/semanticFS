use anyhow::Result;

use crate::db::IndexerDb;

#[cfg(feature = "lancedb")]
const MIN_ROWS_FOR_LANCEDB_VECTOR_INDEX: usize = 65_536;

pub fn sync_vectors_to_lancedb_if_enabled(_db: &IndexerDb, _version: u64) -> Result<()> {
    #[cfg(feature = "lancedb")]
    {
        if vector_backend_enabled() {
            sync_lancedb(_db, _version)?;
        }
    }
    Ok(())
}

#[cfg(feature = "lancedb")]
fn vector_backend_enabled() -> bool {
    std::env::var("SEMANTICFS_VECTOR_BACKEND")
        .map(|v| v.eq_ignore_ascii_case("lancedb"))
        .unwrap_or(true)
}

#[cfg(feature = "lancedb")]
fn sync_lancedb(db: &IndexerDb, version: u64) -> Result<()> {
    use arrow_array::types::Float32Type;
    use arrow_array::{
        FixedSizeListArray, Int32Array, RecordBatch, RecordBatchIterator, StringArray,
    };
    use arrow_schema::{DataType, Field, Schema};
    use lancedb::database::CreateTableMode;
    use lancedb::{connect, Table};
    use std::sync::Arc;

    let rows = db.fetch_vectors_for_version(version)?;
    if rows.is_empty() {
        return Ok(());
    }
    let row_count = rows.len();

    let dim = rows[0].embedding.len().max(1);
    let uri = std::env::var("SEMANTICFS_LANCEDB_URI")
        .unwrap_or_else(|_| "./.semanticfs/lancedb".to_string());

    if let Some(parent) = std::path::Path::new(&uri).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::create_dir_all(&uri)?;

    let uri_for_task = uri.clone();
    run_async(async move {
        let db_conn = connect(&uri_for_task).execute().await?;
        let table_name = format!("chunks_v{}", version);

        let schema = Arc::new(Schema::new(vec![
            Field::new("chunk_id", DataType::Utf8, false),
            Field::new("path", DataType::Utf8, false),
            Field::new("domain_id", DataType::Utf8, false),
            Field::new("start_line", DataType::Int32, false),
            Field::new("end_line", DataType::Int32, false),
            Field::new("file_hash", DataType::Utf8, false),
            Field::new("trust_label", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    dim as i32,
                ),
                true,
            ),
        ]));

        let vectors = rows.iter().map(|r| {
            let mut vals = r.embedding.iter().map(|v| Some(*v)).collect::<Vec<_>>();
            if vals.len() < dim {
                vals.extend(std::iter::repeat_n(Some(0.0), dim - vals.len()));
            } else if vals.len() > dim {
                vals.truncate(dim);
            }
            Some(vals)
        });

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from_iter_values(
                    rows.iter().map(|r| r.chunk_id.clone()),
                )),
                Arc::new(StringArray::from_iter_values(
                    rows.iter().map(|r| r.path.clone()),
                )),
                Arc::new(StringArray::from_iter_values(
                    rows.iter().map(|r| r.domain_id.clone()),
                )),
                Arc::new(Int32Array::from_iter_values(
                    rows.iter().map(|r| r.start_line as i32),
                )),
                Arc::new(Int32Array::from_iter_values(
                    rows.iter().map(|r| r.end_line as i32),
                )),
                Arc::new(StringArray::from_iter_values(
                    rows.iter().map(|r| r.file_hash.clone()),
                )),
                Arc::new(StringArray::from_iter_values(
                    rows.iter().map(|r| r.trust_label.clone()),
                )),
                Arc::new(
                    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                        vectors, dim as i32,
                    ),
                ),
            ],
        )?;

        let reader = RecordBatchIterator::new(vec![Ok(batch)].into_iter(), schema);
        let table: Table = db_conn
            .create_table(&table_name, Box::new(reader))
            .mode(CreateTableMode::Overwrite)
            .execute()
            .await?;

        if row_count >= MIN_ROWS_FOR_LANCEDB_VECTOR_INDEX {
            if let Err(err) = table
                .create_index(&["vector"], lancedb::index::Index::Auto)
                .execute()
                .await
            {
                tracing::warn!(
                    version,
                    row_count,
                    error = %err,
                    "failed to build lancedb vector index; continuing without ANN index"
                );
            }
        } else {
            tracing::info!(
                version,
                row_count,
                min_rows = MIN_ROWS_FOR_LANCEDB_VECTOR_INDEX,
                "skipping lancedb vector index for small dataset"
            );
        }
        Ok::<(), anyhow::Error>(())
    })?;

    tracing::info!(version, uri, row_count, "synced vectors to lancedb");
    Ok(())
}

#[cfg(feature = "lancedb")]
fn run_async<F, T>(fut: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return Ok(tokio::task::block_in_place(|| handle.block_on(fut))?);
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(fut)
}
