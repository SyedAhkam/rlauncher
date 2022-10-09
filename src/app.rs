use wasm_bindgen::prelude::*;
use web_sys::{HtmlInputElement};
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

pub enum Msg {
    QueryChange(String),
}

pub struct App {
    query: String,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            query: String::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::QueryChange(query) => {
                if query != self.query {
                    self.query = query;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <main class="container bg-emerald-800 px-4 py-4 rounded-lg flex flex-col justify-items-stretch">
                <div class="query">
                    <input 
                        type="text" 
                        class="bg-emerald-900 text-white rounded-lg w-full text-[36px]" 
                        oninput={ctx.link().callback(|e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::QueryChange(input.value())
                        })}
                        value={self.query.clone()} 
                    />
                    <p>{self.query.clone()}</p>
                </div>
            </main>
        }
    }
}