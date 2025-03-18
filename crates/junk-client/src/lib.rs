use wasm_bindgen::prelude::*;
use web_sys::console;
use yew::prelude::*;

// Simple component to display a greeting
#[function_component(App)]
fn app() -> Html {
    let counter = use_state(|| 0);

    // Create a click handler to update the counter
    let onclick = {
        let counter = counter.clone();
        Callback::from(move |_| {
            let value = *counter + 1;
            counter.set(value);
            console::log_1(&format!("Counter clicked! New value: {}", value).into());
        })
    };

    html! {
        <div class="app-container">
            <h1>{ "Hello from Yew WASM!" }</h1>
            <p>{ "This is a simple Yew application running in WebAssembly, served by an actix-web backend." }</p>

            <div class="counter">
                <p>{ "You clicked the button " }{ *counter }{ " times!" }</p>
                <button {onclick}>{ "Click me!" }</button>
            </div>
        </div>
    }
}

// Entry point for the WASM application
#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    // Initialize better panic messages for debugging
    console_error_panic_hook::set_once();

    // Initialize logging
    wasm_logger::init(wasm_logger::Config::default());

    // Log a message to the console to verify WASM is running
    console::log_1(&"WASM initialized successfully!".into());

    // Mount the Yew application to the page
    yew::Renderer::<App>::new().render();

    Ok(())
}

