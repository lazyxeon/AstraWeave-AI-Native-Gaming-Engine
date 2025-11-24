pub mod pipeline;

pub use pipeline::{
    RagPipeline, RagConfig, RagDocument, RagContext,
    ConsolidationStrategy, ForgettingStrategy, InjectionStrategy
};
