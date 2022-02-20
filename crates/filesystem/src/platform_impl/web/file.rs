
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::Response;
use js_sys::Uint8Array;

use crate::File;

impl File {
    pub fn exists(&self) -> bool {
        true
    }

    pub fn load(&mut self) {   
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
                bytes.write().unwrap().append(&mut Uint8Array::new(&data).to_vec());
            }
        );
    }

    pub fn save(&self)  {
        eprintln!("Save not implemented for this platform");
    }
}
