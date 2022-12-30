use std::fs;
use std::path::Path;

fn main() {
    let wasm_file_origin_path = Path::new("../test-client/dist/test-client_bg.wasm");
    let wasm_file_dest_path = Path::new("./static/next_rs_wasm_app.wasm");

    let js_file_origin_path = Path::new("../test-client/dist/test-client.js");
    let js_file_dest_path = Path::new("./static/next_rs_js_app.js");

    fs::create_dir_all("./static").unwrap();

    fs::copy(wasm_file_origin_path, wasm_file_dest_path).unwrap();
    fs::copy(js_file_origin_path, js_file_dest_path).unwrap();
}
