use daft_logical_plan::LogicalPlanBuilder;
use eyre::{bail, ensure, Context};
use spark_connect::{relation::RelType, Range, Relation, Tail};
use tracing::warn;

pub fn to_logical_plan(relation: Relation) -> eyre::Result<LogicalPlanBuilder> {
    if let Some(common) = relation.common {
        warn!("Ignoring common metadata for relation: {common:?}; not yet implemented");
    };

    let Some(rel_type) = relation.rel_type else {
        bail!("Relation type is required");
    };

    match rel_type {
        RelType::Range(x) => range(x).wrap_err("Failed to apply range to logical plan"),
        RelType::Tail(x) => tail(*x).wrap_err("Failed to apply tail to logical plan"),
        plan => bail!("Unsupported relation type: {plan:?}"),
    }
}

fn tail(tail: Tail) -> eyre::Result<LogicalPlanBuilder> {
    let Tail { input, limit } = tail;

    let Some(input) = input else {
        bail!("Input is required");
    };

    to_logical_plan(*input)?.li
}

fn range(range: Range) -> eyre::Result<LogicalPlanBuilder> {
    #[cfg(not(feature = "python"))]
    bail!("Range operations require Python feature to be enabled");

    #[cfg(feature = "python")]
    {
        use daft_scan::python::pylib::ScanOperatorHandle;
        use pyo3::prelude::*;
        let Range {
            start,
            end,
            step,
            num_partitions,
        } = range;

        if let Some(partitions) = num_partitions {
            warn!("{partitions} ignored");
        }

        let start = start.unwrap_or(0);

        let step = usize::try_from(step).wrap_err("step must be a positive integer")?;
        ensure!(step > 0, "step must be greater than 0");

        let plan = Python::with_gil(|py| {
            let range_module = PyModule::import_bound(py, "daft.io._range")
                .wrap_err("Failed to import range module")?;

            let range = range_module
                .getattr(pyo3::intern!(py, "RangeScanOperator"))
                .wrap_err("Failed to get range function")?;

            let range = range
                .call1((start, end, step))
                .wrap_err("Failed to create range scan operator")?
                .to_object(py);

            let scan_operator_handle = ScanOperatorHandle::from_python_scan_operator(range, py)?;

            let plan = LogicalPlanBuilder::table_scan(scan_operator_handle.into(), None)?;

            eyre::Result::<_>::Ok(plan)
        })
        .wrap_err("Failed to create range scan")?;

        Ok(plan)
    }
}