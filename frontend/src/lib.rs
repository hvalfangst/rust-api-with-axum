use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod api;
mod components;
mod pages;

use pages::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/frontend.css"/>
        <Title text="API Frontend"/>

        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/login" view=LoginPage/>
                    <Route path="/register" view=RegisterPage/>
                    <Route path="/locations" view=LocationsPage/>
                    <Route path="/empires" view=EmpiresPage/>
                    <Route path="/users" view=UsersPage/>
                </Routes>
            </main>
        </Router>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}