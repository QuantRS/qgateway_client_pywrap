
use futures::StreamExt;
use pyo3::{prelude::*, types::PyTuple};


#[pyclass]
struct Client {
    client: qgateway_client::client::Client,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl Client {
    pub fn send(&mut self, token: String, key: String, value: String) -> PyResult<()> {
        self.client.send(token, key, value);
        Ok(())
    }

    pub fn subscribe(&mut self, token: String, keys: Vec<String>, callback: PyObject, _py: Python) -> PyResult<()> {
        self.runtime.block_on(async {
            let mut rx = self.client.subscribe(token, keys).await;
            while let Some(data) = rx.next().await {
                let args = PyTuple::new(_py, [data.into_py(_py)]);
                callback.call(_py, args, None).unwrap();
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
        client,
        runtime: rt
    }
}

#[pymodule]
fn qgateway_client_pywrap(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(new_client, m)?)?;
    Ok(())
}