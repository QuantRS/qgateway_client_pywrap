
use futures::{StreamExt, channel::mpsc::UnboundedReceiver};
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
        let mut rx = self.runtime.block_on(async {
            self.client.subscribe(token, keys).await
        });

        self.runtime.spawn(async move {
            while let Some(data) = rx.next().await {
                Python::with_gil( |_py| {
                    let args = PyTuple::new(_py, [data.into_py(_py)]);
                    callback.call(_py, args, None).unwrap();
                });
            }
        });
        Ok(())
    }

    pub fn subscribe_advanced(&mut self, token: String, _py: Python) -> SubscribeAdvanced {
        let rx = self.runtime.block_on(async {
            self.client.subscribe(token, Vec::new()).await
        });

        SubscribeAdvanced{
            rx,
            rt: self.runtime.handle().clone()
        }
    }

    pub fn subscribe_blocking(&mut self, token: String, keys: Vec<String>, callback: PyObject, _py: Python) -> PyResult<()> {
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

#[pyclass]
struct SubscribeAdvanced {
    rx: UnboundedReceiver<String>,
    rt: tokio::runtime::Handle
}

#[pymethods]
impl SubscribeAdvanced {
    pub fn next(&mut self) -> PyResult<String> {
        Ok(self.rt.block_on(async {
            self.rx.next().await.unwrap()
        }))
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