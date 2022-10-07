use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[function_component(App)]
pub fn app() -> Html {

    html! {
        <main class="container">
            <h1>{ "Hello, world!" }</h1>
            <p>{ "This is a simple example of a Tauri app with Yew." }</p>
            <p>{ "Learn more about Tauri at " }<a href="https://tauri.studio">{ "https://tauri.studio" }</a></p>
            <p>{ "Learn more about Yew at " }<a href="https://yew.rs">{ "https://yew.rs" }</a></p>
            <p>{ "Learn more about Tauri commands at " }<a href="https://tauri.studio/en/docs/usage/guides/commands">{ "https://tauri.studio/en/docs/usage/guides/commands" }</a></p>
            <p>{ "Learn more about Tauri API at " }<a href="https://tauri.studio/en/docs/api/js">{ "https://tauri.studio/en/docs/api/js" }</a></p>
            <p>{ "Learn more about Yew API at " }<a href="https://docs.rs/yew">{ "https://docs.rs/yew" }</a></p>
        </main>
    }
}
