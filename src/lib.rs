use std::sync::{Arc, Mutex};
use futures::StreamExt;
use pyo3::{prelude::*, types::PyTuple};

#[pyclass]
struct Client {
    client: Arc<futures::lock::Mutex<qgateway_client::client::Client>>,
    runtime: Arc<Mutex<tokio::runtime::Runtime>>,
}

#[pymethods]
impl Client {
    pub fn close(&mut self) -> PyResult<()> {
        let client_clone = self.client.clone();

        let rt = self.runtime.lock().unwrap();
        rt.block_on(async move {
            client_clone.lock().await.close().await;
        });
        Ok(())
    }

    pub fn send(&mut self, token: String, key: String, value: Vec<u8>) -> PyResult<()> {
        let client_clone = self.client.clone();

        let rt = self.runtime.lock().unwrap();
        rt.spawn(async move {
            client_clone.lock().await.send(token, key, value);
        });
        Ok(())
    }

    pub fn subscribe(&mut self, token: String, keys: Vec<String>, callback: PyObject, _py: Python) -> PyResult<()> {
        let client_clone = self.client.clone();

        let rt = self.runtime.lock().unwrap();
        let mut rx = rt.block_on(async move {
            client_clone.lock().await.subscribe(token, keys).await
        });

        rt.spawn(async move {
            while let Some(data) = rx.next().await {
                Python::with_gil( |_py| {
                    callback.call(_py, PyTuple::new(_py, vec![data]), None).unwrap();
                });
            }
        });
        Ok(())
    }
}

#[pyfunction]
fn new_client(url: String, auth_token: String) -> Client {
    let mut client = qgateway_client::client::Client::new(url, auth_token);

    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    rt.block_on(async {
        client.connection().await;
    });

    Client{
        client:Arc::new(futures::lock::Mutex::new(client)),
        runtime:Arc::new(Mutex::new(rt))
    }
}

#[pymodule]
fn qgateway_client_pywrap(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(new_client, m)?)?;
    Ok(())
}