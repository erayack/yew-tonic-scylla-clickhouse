mod api;

use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

const BACKEND_URL: &str = match option_env!("FRONTEND_BACKEND_URL") {
    Some(value) => value,
    None => "http://127.0.0.1:50051",
};

#[function_component(App)]
fn app() -> Html {
    let name = use_state(String::new);
    let payload = use_state(String::new);
    let status = use_state(|| None::<String>);
    let loading = use_state(|| false);

    {
        let status = status.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match api::health_check(BACKEND_URL).await {
                    Ok(value) => status.set(Some(format!("Backend health: {value}"))),
                    Err(error) => status.set(Some(format!("Backend health check failed: {error}"))),
                }
            });
            || ()
        });
    }

    let on_name_input = {
        let name = name.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            name.set(input.value());
        })
    };

    let on_payload_input = {
        let payload = payload.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlTextAreaElement = event.target_unchecked_into();
            payload.set(input.value());
        })
    };

    let onsubmit = {
        let name = name.clone();
        let payload = payload.clone();
        let status = status.clone();
        let loading = loading.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            if *loading {
                return;
            }
            loading.set(true);
            status.set(Some("Creating event...".to_string()));
            let name_value = (*name).clone();
            let payload_value = (*payload).clone();
            let status = status.clone();
            let loading = loading.clone();
            spawn_local(async move {
                match api::create_event(BACKEND_URL, name_value, payload_value).await {
                    Ok(id) => status.set(Some(format!("Created event {id}"))),
                    Err(error) => status.set(Some(format!("Create event failed: {error}"))),
                }
                loading.set(false);
            });
        })
    };

    html! {
        <main style="max-width: 720px; margin: 2rem auto; font-family: system-ui, sans-serif;">
            <h1>{"Yew + tonic + ScyllaDB + ClickHouse"}</h1>
            <p>{"A minimal starter template with a Rust/WASM frontend and tonic backend."}</p>
            <form {onsubmit}>
                <label for="event-name">{"Event name"}</label>
                <input id="event-name" type="text" value={(*name).clone()} oninput={on_name_input} placeholder="page_view" />
                <label for="event-payload" style="display:block; margin-top: 1rem;">{"Payload"}</label>
                <textarea id="event-payload" value={(*payload).clone()} oninput={on_payload_input} placeholder="{\"path\":\"/\"}" />
                <div style="margin-top: 1rem;">
                    <button type="submit" disabled={*loading}>{ if *loading { "Sending..." } else { "Create event" } }</button>
                </div>
            </form>
            if let Some(message) = &*status {
                <p>{message}</p>
            }
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
