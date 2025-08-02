use leptos::*;
use crate::api::{self, Location, Empire, User, UpsertLocation, UpsertEmpire, UpsertUser};

#[component]
pub fn LoginForm() -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(false);

    let login_action = create_action(move |(email, password): &(String, String)| {
        let email = email.clone();
        let password = password.clone();
        async move {
            set_loading.set(true);
            set_error.set(None);
            
            match api::login(email, password).await {
                Ok(_) => {
                    // Redirect to home page
                    let window = web_sys::window().unwrap();
                    window.location().set_href("/").unwrap();
                },
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        login_action.dispatch((email.get(), password.get()));
    };

    view! {
        <div class="form-container">
            <h2>"Login"</h2>
            <form on:submit=on_submit>
                <div class="form-group">
                    <label for="email">"Email:"</label>
                    <input
                        type="email"
                        id="email"
                        required
                        prop:value=email
                        on:input=move |ev| set_email.set(event_target_value(&ev))
                    />
                </div>
                
                <div class="form-group">
                    <label for="password">"Password:"</label>
                    <input
                        type="password"
                        id="password"
                        required
                        prop:value=password
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                    />
                </div>
                
                {move || error.get().map(|e| view! {
                    <div class="error">{e}</div>
                })}
                
                <button type="submit" disabled=move || loading.get()>
                    {move || if loading.get() { "Logging in..." } else { "Login" }}
                </button>
            </form>
        </div>
    }
}

#[component]
pub fn RegisterForm() -> impl IntoView {
    let (fullname, set_fullname) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (role, set_role) = create_signal("READER".to_string());
    let (error, set_error) = create_signal(None::<String>);
    let (success, set_success) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(false);

    let register_action = create_action(move |(fullname, email, password, role): &(String, String, String, String)| {
        let fullname = fullname.clone();
        let email = email.clone();
        let password = password.clone();
        let role = role.clone();
        async move {
            set_loading.set(true);
            set_error.set(None);
            set_success.set(None);
            
            match api::register(fullname, email, password, role).await {
                Ok(_) => {
                    set_success.set(Some("Registration successful! You can now log in.".to_string()));
                },
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        register_action.dispatch((fullname.get(), email.get(), password.get(), role.get()));
    };

    view! {
        <div class="form-container">
            <h2>"Register"</h2>
            <form on:submit=on_submit>
                <div class="form-group">
                    <label for="fullname">"Full Name:"</label>
                    <input
                        type="text"
                        id="fullname"
                        required
                        prop:value=fullname
                        on:input=move |ev| set_fullname.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label for="email">"Email:"</label>
                    <input
                        type="email"
                        id="email"
                        required
                        prop:value=email
                        on:input=move |ev| set_email.set(event_target_value(&ev))
                    />
                </div>
                
                <div class="form-group">
                    <label for="password">"Password:"</label>
                    <input
                        type="password"
                        id="password"
                        required
                        prop:value=password
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label for="role">"Role:"</label>
                    <select
                        id="role"
                        prop:value=role
                        on:change=move |ev| set_role.set(event_target_value(&ev))
                    >
                        <option value="READER">"Reader"</option>
                        <option value="WRITER">"Writer"</option>
                        <option value="EDITOR">"Editor"</option>
                        <option value="ADMIN">"Admin"</option>
                    </select>
                </div>
                
                {move || error.get().map(|e| view! {
                    <div class="error">{e}</div>
                })}
                
                {move || success.get().map(|s| view! {
                    <div class="success">{s}</div>
                })}
                
                <button type="submit" disabled=move || loading.get()>
                    {move || if loading.get() { "Registering..." } else { "Register" }}
                </button>
            </form>
        </div>
    }
}

#[component]
pub fn LocationForm(
    location: Option<Location>,
    on_submit: WriteSignal<Option<UpsertLocation>>,
    on_cancel: WriteSignal<bool>
) -> impl IntoView {
    let (star_system, set_star_system) = create_signal(
        location.as_ref().map(|l| l.star_system.clone()).unwrap_or_default()
    );
    let (area, set_area) = create_signal(
        location.as_ref().map(|l| l.area.clone()).unwrap_or_default()
    );

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        let location = UpsertLocation {
            star_system: star_system.get(),
            area: area.get(),
        };
        
        on_submit.set(Some(location));
    };

    let handle_cancel = move |_| {
        on_cancel.set(true);
    };

    view! {
        <div class="form-container">
            <h3>{if location.is_some() { "Edit Location" } else { "Add Location" }}</h3>
            <form on:submit=handle_submit>
                <div class="form-group">
                    <label for="star_system">"Star System:"</label>
                    <input
                        type="text"
                        id="star_system"
                        required
                        prop:value=star_system
                        on:input=move |ev| set_star_system.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label for="area">"Area:"</label>
                    <input
                        type="text"
                        id="area"
                        required
                        prop:value=area
                        on:input=move |ev| set_area.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-actions">
                    <button type="submit">
                        {if location.is_some() { "Update" } else { "Create" }}
                    </button>
                    <button type="button" on:click=handle_cancel>"Cancel"</button>
                </div>
            </form>
        </div>
    }
}

#[component]
pub fn EmpireForm(
    empire: Option<Empire>,
    on_submit: WriteSignal<Option<UpsertEmpire>>,
    on_cancel: WriteSignal<bool>
) -> impl IntoView {
    let (name, set_name) = create_signal(
        empire.as_ref().map(|e| e.name.clone()).unwrap_or_default()
    );
    let (slogan, set_slogan) = create_signal(
        empire.as_ref().map(|e| e.slogan.clone()).unwrap_or_default()
    );
    let (location_id, set_location_id) = create_signal(
        empire.as_ref().map(|e| e.location_id.to_string()).unwrap_or_else(|| "1".to_string())
    );
    let (description, set_description) = create_signal(
        empire.as_ref().map(|e| e.description.clone()).unwrap_or_default()
    );

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        let location_id = location_id.get().parse::<i32>().unwrap_or(1);
        
        let empire = UpsertEmpire {
            name: name.get(),
            slogan: slogan.get(),
            location_id,
            description: description.get(),
        };
        
        on_submit.set(Some(empire));
    };

    let handle_cancel = move |_| {
        on_cancel.set(true);
    };

    view! {
        <div class="form-container">
            <h3>{if empire.is_some() { "Edit Empire" } else { "Add Empire" }}</h3>
            <form on:submit=handle_submit>
                <div class="form-group">
                    <label for="name">"Name:"</label>
                    <input
                        type="text"
                        id="name"
                        required
                        prop:value=name
                        on:input=move |ev| set_name.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label for="slogan">"Slogan:"</label>
                    <input
                        type="text"
                        id="slogan"
                        required
                        prop:value=slogan
                        on:input=move |ev| set_slogan.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label for="location_id">"Location ID:"</label>
                    <input
                        type="number"
                        id="location_id"
                        required
                        prop:value=location_id
                        on:input=move |ev| set_location_id.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label for="description">"Description:"</label>
                    <textarea
                        id="description"
                        required
                        prop:value=description
                        on:input=move |ev| set_description.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-actions">
                    <button type="submit">
                        {if empire.is_some() { "Update" } else { "Create" }}
                    </button>
                    <button type="button" on:click=handle_cancel>"Cancel"</button>
                </div>
            </form>
        </div>
    }
}

#[component]
pub fn UserForm(
    user: Option<User>,
    on_submit: WriteSignal<Option<UpsertUser>>,
    on_cancel: WriteSignal<bool>
) -> impl IntoView {
    let (fullname, set_fullname) = create_signal(
        user.as_ref().map(|u| u.fullname.clone()).unwrap_or_default()
    );
    let (email, set_email) = create_signal(
        user.as_ref().map(|u| u.email.clone()).unwrap_or_default()
    );
    let (password, set_password) = create_signal(String::new());
    let (role, set_role) = create_signal(
        user.as_ref().map(|u| u.role.clone()).unwrap_or_else(|| "READER".to_string())
    );

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        let user_data = UpsertUser {
            fullname: fullname.get(),
            email: email.get(),
            password: password.get(),
            role: role.get(),
        };
        
        on_submit.set(Some(user_data));
    };

    let handle_cancel = move |_| {
        on_cancel.set(true);
    };

    view! {
        <div class="form-container">
            <h3>{if user.is_some() { "Edit User" } else { "Add User" }}</h3>
            <form on:submit=handle_submit>
                <div class="form-group">
                    <label for="fullname">"Full Name:"</label>
                    <input
                        type="text"
                        id="fullname"
                        required
                        prop:value=fullname
                        on:input=move |ev| set_fullname.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label for="email">"Email:"</label>
                    <input
                        type="email"
                        id="email"
                        required
                        prop:value=email
                        on:input=move |ev| set_email.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label for="password">"Password:"</label>
                    <input
                        type="password"
                        id="password"
                        required=user.is_none()
                        placeholder=if user.is_some() { "Leave blank to keep current password" } else { "" }
                        prop:value=password
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-group">
                    <label for="role">"Role:"</label>
                    <select
                        id="role"
                        prop:value=role
                        on:change=move |ev| set_role.set(event_target_value(&ev))
                    >
                        <option value="READER">"Reader"</option>
                        <option value="WRITER">"Writer"</option>
                        <option value="EDITOR">"Editor"</option>
                        <option value="ADMIN">"Admin"</option>
                    </select>
                </div>

                <div class="form-actions">
                    <button type="submit">
                        {if user.is_some() { "Update" } else { "Create" }}
                    </button>
                    <button type="button" on:click=handle_cancel>"Cancel"</button>
                </div>
            </form>
        </div>
    }
}