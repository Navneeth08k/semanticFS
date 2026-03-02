pub mod audit;
pub mod config;
pub mod embedding;
pub mod health;
pub mod retrieval;
pub mod runtime;
pub mod version;

pub use audit::AuditEvent;
pub use config::{
    DomainContractError, EmbeddingConfig, FilterConfig, FuseCacheConfig, FuseSessionConfig,
    IndexConfig, MapConfig, McpConfig, ObservabilityConfig, PolicyConfig, RetrievalConfig,
    SemanticFsConfig, WorkspaceConfig, WorkspaceDomainConfig, WorkspaceDomainPlan,
    WorkspaceDomainReport,
};
pub use embedding::{cosine_similarity, embed_text_hash};
pub use health::{HealthIndex, HealthReport};
pub use retrieval::{GroundedHit, HitSource, TrustLevel};
pub use runtime::IndexingStatus;
pub use version::{IndexVersionRecord, IndexVersionState};
