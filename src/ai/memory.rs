//! Módulo de memoria vectorial usando RIG y SQLite

use anyhow::Result;
use rig::embeddings::EmbeddingModel;
use rig::vector_store::VectorSearchRequest;
use rig::vector_store::VectorStoreIndex;
use rig_sqlite::{Column, SqliteVectorStore, SqliteVectorStoreTable};
use rusqlite::Error as RusqliteError;
use tokio::sync::RwLock;
use tokio_rusqlite::{Connection, Error as TokioSqliteError};
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct NoteDocument {
    pub id: String,
    pub content: String,
    #[serde(flatten)]
    pub metadata: serde_json::Value,
}

impl rig_sqlite::SqliteVectorStoreTable for NoteDocument {
    fn name() -> &'static str {
        "rig_note"
    }

    fn schema() -> Vec<Column> {
        vec![
            Column::new("id", "TEXT PRIMARY KEY"),
            Column::new("content", "TEXT"),
            Column::new("metadata", "TEXT"),
        ]
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn column_values(&self) -> Vec<(&'static str, Box<dyn rig_sqlite::ColumnValue>)> {
        vec![
            ("id", Box::new(self.id.clone())),
            ("content", Box::new(self.content.clone())),
            (
                "metadata",
                Box::new(serde_json::to_string(&self.metadata).unwrap_or_default()),
            ),
        ]
    }
}

pub struct NoteMemory<M: EmbeddingModel + Sync + Send + Clone + 'static> {
    store: RwLock<SqliteVectorStore<M, NoteDocument>>,
    embedding_model: M,
    conn: Connection,
}

impl<M: EmbeddingModel + Sync + Send + Clone + 'static> std::fmt::Debug for NoteMemory<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoteMemory")
            .field("store", &"SqliteVectorStore")
            .field("embedding_model", &"EmbeddingModel")
            .field("conn", &"Connection")
            .finish()
    }
}

impl<M: EmbeddingModel + Sync + Send + Clone + 'static> NoteMemory<M> {
    pub async fn new(db_path: &str, embedding_model: M) -> Result<Self> {
        // Initialize sqlite-vec extension BEFORE opening the connection
        unsafe {
            use rusqlite::ffi::{sqlite3, sqlite3_api_routines, sqlite3_auto_extension};
            use sqlite_vec::sqlite3_vec_init;

            type SqliteExtensionFn = unsafe extern "C" fn(
                *mut sqlite3,
                *mut *mut i8,
                *const sqlite3_api_routines,
            ) -> i32;

            sqlite3_auto_extension(Some(std::mem::transmute::<*const (), SqliteExtensionFn>(
                sqlite3_vec_init as *const (),
            )));
        }

        let conn = Connection::open(db_path).await?;
        let store_conn = conn.clone();
        let store = SqliteVectorStore::new(store_conn, &embedding_model).await?;

        Ok(Self {
            store: RwLock::new(store),
            embedding_model,
            conn,
        })
    }

    /// Clear all indexed notes - useful for reindexing from scratch
    pub async fn clear_all(&self) -> Result<()> {
        eprintln!("🗑️ [NoteMemory::clear_all] Limpiando todas las notas indexadas...");

        let base_table = NoteDocument::name().to_string();
        let embeddings_prefix = format!("{}_embeddings%", NoteDocument::name());

        self.conn
            .call(move |conn| {
                let mut stmt = conn
                    .prepare("SELECT name FROM sqlite_master WHERE name LIKE ?1")
                    .map_err(TokioSqliteError::from)?;
                let pattern = embeddings_prefix.clone();
                let mut tables = stmt
                    .query_map([pattern.as_str()], |row| row.get::<_, String>(0))
                    .map_err(TokioSqliteError::from)?
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(TokioSqliteError::from)?;

                // Ensure we drop the primary virtual table and base table as well
                tables.push(format!("{}_embeddings", NoteDocument::name()));
                tables.sort();
                tables.dedup();

                for table in tables {
                    let sql = format!("DROP TABLE IF EXISTS {}", table);
                    conn.execute(&sql, []).map_err(TokioSqliteError::from)?;
                }

                conn.execute(&format!("DROP TABLE IF EXISTS {}", base_table), [])
                    .map_err(TokioSqliteError::from)?;

                eprintln!("   ✅ Tablas eliminadas");
                Ok::<_, TokioSqliteError>(())
            })
            .await?;

        // Re-create the vector store tables after dropping everything
        let new_store = SqliteVectorStore::new(self.conn.clone(), &self.embedding_model).await?;
        *self.store.write().await = new_store;

        Ok(())
    }

    pub async fn index_note(
        &self,
        note_id: &str,
        content: &str,
        metadata: serde_json::Value,
    ) -> Result<()> {
        eprintln!(
            "🔍 [NoteMemory::index_note] Iniciando indexación de: {}",
            note_id
        );

        // Truncate content to avoid context length limits
        let truncated_content = if content.len() > 25000 {
            eprintln!(
                "⚠️ Contenido truncado de {} a 25000 caracteres",
                content.len()
            );
            &content[..25000]
        } else {
            content
        };

        eprintln!("   Contenido: {} chars", truncated_content.len());

        // Generate embedding
        let embedding = match self.embedding_model.embed_text(truncated_content).await {
            Ok(emb) => {
                eprintln!("✅ Embedding generado: {} dimensiones", emb.vec.len());
                emb
            }
            Err(e) => {
                eprintln!("❌ Error generando embedding: {}", e);
                return Err(anyhow::anyhow!("Error generando embedding: {}", e));
            }
        };

        let doc = NoteDocument {
            id: note_id.to_string(),
            content: truncated_content.to_string(),
            metadata,
        };

        eprintln!("🔍 Insertando documento en SQLite...");
        let store = self.store.read().await;
        let result = store
            .add_rows(vec![(doc, rig::OneOrMany::one(embedding))])
            .await;
        drop(store);

        match result {
            Ok(_) => {
                eprintln!("✅ Documento insertado exitosamente");
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Error insertando documento: {}", e);
                Err(anyhow::anyhow!("Error insertando documento: {}", e))
            }
        }
    }

    pub async fn remove_note(&self, note_id: &str) -> Result<()> {
        eprintln!(
            "🗑️ [NoteMemory::remove_note] Intentando eliminar: {}",
            note_id
        );
        let id = note_id.to_string();
        self.conn
            .call(move |conn| {
                // First, try to get the rowid
                let rowid: Option<i64> = conn
                    .query_row(
                        "SELECT rowid FROM rig_note WHERE id = ?",
                        [id.as_str()],
                        |row| row.get(0),
                    )
                    .ok();

                if let Some(rid) = rowid {
                    eprintln!("   Encontrado rowid: {}, eliminando embeddings...", rid);
                    // Delete from embeddings table manually
                    match conn.execute("DELETE FROM rig_note_embeddings WHERE rowid = ?", [rid]) {
                        Ok(n) => eprintln!("   ✅ Eliminadas {} filas de embeddings", n),
                        Err(e) => eprintln!("   ⚠️ Error eliminando embeddings: {}", e),
                    }
                } else {
                    eprintln!("   ℹ️ No se encontró nota con id: {}", id);
                }

                // Delete from main table
                match conn.execute("DELETE FROM rig_note WHERE id = ?", [id.as_str()]) {
                    Ok(n) => {
                        eprintln!("   ✅ Eliminadas {} filas de rig_note", n);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("   ❌ Error eliminando de rig_note: {}", e);
                        Err(TokioSqliteError::Rusqlite(e))
                    }
                }
            })
            .await?;
        Ok(())
    }

    pub async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(f32, String, serde_json::Value, String)>> {
        eprintln!(
            "🔍 [NoteMemory::search] Buscando: '{}' (limit: {})",
            query, limit
        );

        // Create an index from the store (requires cloning store and model)
        let store = self.store.read().await;
        let store_clone = store.clone();
        drop(store);

        let index = store_clone.index(self.embedding_model.clone());

        let request = VectorSearchRequest::builder()
            .query(query.to_string())
            .samples(limit as u64)
            .build()?;

        let results = index.top_n::<NoteDocument>(request).await?;

        eprintln!("   ✅ Encontrados {} resultados brutos", results.len());
        for (score, id, _) in &results {
            eprintln!("      - {} (score: {})", id, score);
        }

        let mut mapped_results = Vec::new();
        for (score, id, doc) in results {
            mapped_results.push((score as f32, id, doc.metadata, doc.content));
        }

        Ok(mapped_results)
    }
}
