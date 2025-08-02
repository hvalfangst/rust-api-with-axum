use leptos::*;
use leptos_router::*;
use crate::api;

#[component]
pub fn Navbar() -> impl IntoView {
    let (is_logged_in, set_is_logged_in) = create_signal(api::get_token().is_some());
    
    let logout = move |_| {
        api::clear_token();
        set_is_logged_in.set(false);
    };

    view! {
        <nav class="navbar">
            <div class="nav-brand">
                <A href="/">"API Frontend"</A>
            </div>
            <div class="nav-links">
                <A href="/">"Home"</A>
                {move || if is_logged_in.get() {
                    view! {
                        <A href="/locations">"Locations"</A>
                        <A href="/empires">"Empires"</A>
                        <A href="/users">"Users"</A>
                        <button on:click=logout class="logout-btn">"Logout"</button>
                    }.into_view()
                } else {
                    view! {
                        <A href="/login">"Login"</A>
                        <A href="/register">"Register"</A>
                    }.into_view()
                }}
            </div>
        </nav>
    }
}