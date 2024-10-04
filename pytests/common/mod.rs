use std::{thread, time::Duration};

use pyo3::prelude::*;
use pyo3_async_runtimes::TaskLocals;

pub(super) const TEST_MOD: &'static str = r#"
import asyncio

async def py_sleep(duration):
    await asyncio.sleep(duration)

async def sleep_for_1s(sleep_for):
    await sleep_for(1)
"#;

pub(super) async fn test_into_future(event_loop: PyObject) -> PyResult<()> {
    let fut = Python::with_gil(|py| {
        let test_mod =
            PyModule::from_code_bound(py, TEST_MOD, "test_rust_coroutine/test_mod.py", "test_mod")?;

        pyo3_async_runtimes::into_future_with_locals(
            &TaskLocals::new(event_loop.into_bound(py)),
            test_mod.call_method1("py_sleep", (1.into_py(py),))?,
        )
    })?;

    fut.await?;

    Ok(())
}

pub(super) fn test_blocking_sleep() -> PyResult<()> {
    thread::sleep(Duration::from_secs(1));
    Ok(())
}

pub(super) async fn test_other_awaitables(event_loop: PyObject) -> PyResult<()> {
    let fut = Python::with_gil(|py| {
        let functools = py.import_bound("functools")?;
        let time = py.import_bound("time")?;

        // spawn a blocking sleep in the threadpool executor - returns a task, not a coroutine
        let task = event_loop.bind(py).call_method1(
            "run_in_executor",
            (
                py.None(),
                functools.call_method1("partial", (time.getattr("sleep")?, 1))?,
            ),
        )?;

        pyo3_async_runtimes::into_future_with_locals(
            &TaskLocals::new(event_loop.into_bound(py)),
            task,
        )
    })?;

    fut.await?;

    Ok(())
}
