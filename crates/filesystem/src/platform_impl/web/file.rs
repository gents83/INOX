
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::Response;
use js_sys::Uint8Array;

use crate::File;

impl File {
    pub fn exists(&self) -> bool {
        true
    }

    pub fn load<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Vec<u8>) + 'static, {
        let bytes = self.bytes.clone();
        let filepath = self.path.to_str().unwrap().to_string();
        wasm_bindgen_futures::spawn_local(
            async move {
                let window = web_sys::window().unwrap();
                let resp_value = JsFuture::from(window.fetch_with_str(filepath.as_str()))
                    .await
                    .unwrap();
                let resp: Response = resp_value.dyn_into().unwrap();
                let data = JsFuture::from(resp.array_buffer().unwrap()).await.unwrap();
                let mut bytes = bytes.write().unwrap();
                bytes.append(&mut Uint8Array::new(&data).to_vec());
                f(&mut bytes);
            }
        );
    }

    pub fn save<F>(&mut self, _f: F)
    where
        F: FnMut(&mut Vec<u8>) + 'static,
    {
        eprintln!("Save not implemented for this platform");
    }
}
