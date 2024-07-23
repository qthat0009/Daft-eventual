use std::sync::Arc;

use common_error::DaftResult;
use daft_dsl::ExprRef;
use daft_micropartition::MicroPartition;
use tracing::instrument;

use super::intermediate_op::IntermediateOperator;

#[derive(Clone)]
pub struct FilterOperator {
    predicate: ExprRef,
}

impl FilterOperator {
    pub fn new(predicate: ExprRef) -> Self {
        Self { predicate }
    }
}

impl IntermediateOperator for FilterOperator {
    #[instrument(skip_all, name = "FilterOperator::execute")]
    fn execute(&self, input: &Arc<MicroPartition>) -> DaftResult<Arc<MicroPartition>> {
        input.filter(&[self.predicate.clone()]).map(Arc::new)
    }

    fn name(&self) -> &'static str {
        "FilterOperator"
    }
}
