use leptos::*;
use leptos_router::*;
use crate::api;
use crate::api::{Location as ApiLocation, Empire as ApiEmpire, User as ApiUser, UpsertLocation, UpsertEmpire, UpsertUser, is_authenticated};
use crate::components::navbar::Navbar;
use crate::components::forms::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Navbar/>
        <div class="container">
            <h1>"Welcome to the API Frontend"</h1>
            <p>"This is a Leptos frontend for the Axum API with authentication."</p>
            
            {move || if api::get_token().is_some() {
                view! {
                    <div class="dashboard">
                        <h2>"Dashboard"</h2>
                        <div class="dashboard-links">
                            <A href="/locations" class="dashboard-link">"Manage Locations"</A>
                            <A href="/empires" class="dashboard-link">"Manage Empires"</A>
                            <A href="/users" class="dashboard-link">"Manage Users"</A>
                        </div>
                    </div>
                }.into_view()
            } else {
                view! {
                    <div class="auth-prompt">
                        <p>"Please log in to access the dashboard."</p>
                        <A href="/login" class="btn btn-primary">"Login"</A>
                        <A href="/register" class="btn btn-secondary">"Register"</A>
                    </div>
                }.into_view()
            }}
        </div>
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <Navbar/>
        <div class="container">
            <LoginForm/>
            <p class="auth-switch">
                "Don't have an account? "
                <A href="/register">"Register here"</A>
            </p>
        </div>
    }
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    view! {
        <Navbar/>
        <div class="container">
            <RegisterForm/>
            <p class="auth-switch">
                "Already have an account? "
                <A href="/login">"Login here"</A>
            </p>
        </div>
    }
}

#[component]
pub fn LocationsPage() -> impl IntoView {
    let (locations, set_locations) = create_signal(Vec::<ApiLocation>::new());
    let (show_form, set_show_form) = create_signal(false);
    let (editing_location, set_editing_location) = create_signal(None::<ApiLocation>);
    let (form_data, set_form_data) = create_signal(None::<UpsertLocation>);
    let (cancel_form, set_cancel_form) = create_signal(false);
    let (error, set_error) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(false);
    let (auth_state, set_auth_state) = create_signal(is_authenticated());

    // Update auth state reactively
    create_effect(move |_| {
        set_auth_state.set(is_authenticated());
    });

    // Load locations on mount
    create_effect(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            match api::get_locations().await {
                Ok(locs) => set_locations.set(locs),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    // Handle form submission
    create_effect(move |_| {
        if let Some(data) = form_data.get() {
            spawn_local(async move {
                set_loading.set(true);
                let result = if let Some(location) = editing_location.get() {
                    api::update_location(location.id, data).await
                } else {
                    api::create_location(data).await
                };

                match result {
                    Ok(_) => {
                        // Reload locations
                        match api::get_locations().await {
                            Ok(locs) => set_locations.set(locs),
                            Err(e) => set_error.set(Some(e)),
                        }
                        set_show_form.set(false);
                        set_editing_location.set(None);
                    },
                    Err(e) => set_error.set(Some(e)),
                }
                set_loading.set(false);
            });
            set_form_data.set(None);
        }
    });

    // Handle form cancel
    create_effect(move |_| {
        if cancel_form.get() {
            set_show_form.set(false);
            set_editing_location.set(None);
            set_cancel_form.set(false);
        }
    });

    let add_location = move |_| {
        set_editing_location.set(None);
        set_show_form.set(true);
    };

    let edit_location = move |location: ApiLocation| {
        set_editing_location.set(Some(location));
        set_show_form.set(true);
    };

    let delete_location_action = move |id: i32| {
        spawn_local(async move {
            set_loading.set(true);
            match api::delete_location(id).await {
                Ok(_) => {
                    match api::get_locations().await {
                        Ok(locs) => set_locations.set(locs),
                        Err(e) => set_error.set(Some(e)),
                    }
                },
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    };

    view! {
        <Navbar/>
        <div class="container">
            <h1>"Locations"</h1>
            
            {move || error.get().map(|e| view! {
                <div class="error">{e}</div>
            })}

            {move || if show_form.get() {
                view! {
                    <LocationForm
                        location=editing_location.get()
                        on_submit=set_form_data
                        on_cancel=set_cancel_form
                    />
                }.into_view()
            } else {
                view! {
                    <div class="actions">
                        {move || if auth_state.get() {
                            view! {
                                <button on:click=add_location class="btn btn-primary">"Add Location"</button>
                            }.into_view()
                        } else {
                            view! {
                                <button disabled class="btn btn-secondary" title="Please log in to add locations">"Add Location"</button>
                            }.into_view()
                        }}
                    </div>

                    <div class="data-table">
                        {move || if loading.get() {
                            view! { <div class="loading">"Loading..."</div> }.into_view()
                        } else {
                            view! {
                                <table>
                                    <thead>
                                        <tr>
                                            <th>"ID"</th>
                                            <th>"Star System"</th>
                                            <th>"Area"</th>
                                            <th>"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || locations.get()
                                            key=|location| location.id
                                            children=move |location| {
                                                let edit_loc = std::rc::Rc::new(location.clone());
                                                let delete_id = location.id;
                                                let edit_loc_clone = edit_loc.clone();
                                                view! {
                                                    <tr>
                                                        <td>{location.id}</td>
                                                        <td>{location.star_system}</td>
                                                        <td>{location.area}</td>
                                                        <td class="actions">
                                                            <Show
                                                                when=move || auth_state.get()
                                                                fallback=move || view! {
                                                                    <button disabled class="btn btn-small btn-secondary" title="Please log in to edit">"Edit"</button>
                                                                    <button disabled class="btn btn-small btn-secondary" title="Please log in to delete">"Delete"</button>
                                                                }
                                                            >
                                                                <button 
                                                                    on:click={
                                                                        let edit_loc = edit_loc_clone.clone();
                                                                        move |_| edit_location((*edit_loc).clone())
                                                                    }
                                                                    class="btn btn-small btn-secondary"
                                                                >
                                                                    "Edit"
                                                                </button>
                                                                <button 
                                                                    on:click=move |_| delete_location_action(delete_id)
                                                                    class="btn btn-small btn-danger"
                                                                >
                                                                    "Delete"
                                                                </button>
                                                            </Show>
                                                        </td>
                                                    </tr>
                                                }
                                            }
                                        />
                                    </tbody>
                                </table>
                            }.into_view()
                        }}
                    </div>
                }.into_view()
            }}
        </div>
    }
}

#[component]
pub fn EmpiresPage() -> impl IntoView {
    let (empires, set_empires) = create_signal(Vec::<ApiEmpire>::new());
    let (show_form, set_show_form) = create_signal(false);
    let (editing_empire, set_editing_empire) = create_signal(None::<ApiEmpire>);
    let (form_data, set_form_data) = create_signal(None::<UpsertEmpire>);
    let (cancel_form, set_cancel_form) = create_signal(false);
    let (error, set_error) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(false);
    let (auth_state, set_auth_state) = create_signal(is_authenticated());

    // Update auth state reactively
    create_effect(move |_| {
        set_auth_state.set(is_authenticated());
    });

    // Load empires on mount
    create_effect(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            match api::get_empires().await {
                Ok(emps) => set_empires.set(emps),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    // Handle form submission
    create_effect(move |_| {
        if let Some(data) = form_data.get() {
            spawn_local(async move {
                set_loading.set(true);
                let result = if let Some(empire) = editing_empire.get() {
                    api::update_empire(empire.id, data).await
                } else {
                    api::create_empire(data).await
                };

                match result {
                    Ok(_) => {
                        // Reload empires
                        match api::get_empires().await {
                            Ok(emps) => set_empires.set(emps),
                            Err(e) => set_error.set(Some(e)),
                        }
                        set_show_form.set(false);
                        set_editing_empire.set(None);
                    },
                    Err(e) => set_error.set(Some(e)),
                }
                set_loading.set(false);
            });
            set_form_data.set(None);
        }
    });

    // Handle form cancel
    create_effect(move |_| {
        if cancel_form.get() {
            set_show_form.set(false);
            set_editing_empire.set(None);
            set_cancel_form.set(false);
        }
    });

    let add_empire = move |_| {
        set_editing_empire.set(None);
        set_show_form.set(true);
    };

    let edit_empire = move |empire: ApiEmpire| {
        set_editing_empire.set(Some(empire));
        set_show_form.set(true);
    };

    let delete_empire_action = move |id: i32| {
        spawn_local(async move {
            set_loading.set(true);
            match api::delete_empire(id).await {
                Ok(_) => {
                    match api::get_empires().await {
                        Ok(emps) => set_empires.set(emps),
                        Err(e) => set_error.set(Some(e)),
                    }
                },
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    };

    view! {
        <Navbar/>
        <div class="container">
            <h1>"Empires"</h1>
            
            {move || error.get().map(|e| view! {
                <div class="error">{e}</div>
            })}

            {move || if show_form.get() {
                view! {
                    <EmpireForm
                        empire=editing_empire.get()
                        on_submit=set_form_data
                        on_cancel=set_cancel_form
                    />
                }.into_view()
            } else {
                view! {
                    <div class="actions">
                        {move || if auth_state.get() {
                            view! {
                                <button on:click=add_empire class="btn btn-primary">"Add Empire"</button>
                            }.into_view()
                        } else {
                            view! {
                                <button disabled class="btn btn-secondary" title="Please log in to add empires">"Add Empire"</button>
                            }.into_view()
                        }}
                    </div>

                    <div class="data-table">
                        {move || if loading.get() {
                            view! { <div class="loading">"Loading..."</div> }.into_view()
                        } else {
                            view! {
                                <table>
                                    <thead>
                                        <tr>
                                            <th>"ID"</th>
                                            <th>"Name"</th>
                                            <th>"Slogan"</th>
                                            <th>"Location ID"</th>
                                            <th>"Description"</th>
                                            <th>"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || empires.get()
                                            key=|empire| empire.id
                                            children=move |empire| {
                                                let edit_emp = std::rc::Rc::new(empire.clone());
                                                let delete_id = empire.id;
                                                let edit_emp_clone = edit_emp.clone();
                                                view! {
                                                    <tr>
                                                        <td>{empire.id}</td>
                                                        <td>{empire.name}</td>
                                                        <td>{empire.slogan}</td>
                                                        <td>{empire.location_id}</td>
                                                        <td>{empire.description}</td>
                                                        <td class="actions">
                                                            <Show
                                                                when=move || auth_state.get()
                                                                fallback=move || view! {
                                                                    <button disabled class="btn btn-small btn-secondary" title="Please log in to edit">"Edit"</button>
                                                                    <button disabled class="btn btn-small btn-secondary" title="Please log in to delete">"Delete"</button>
                                                                }
                                                            >
                                                                <button 
                                                                    on:click={
                                                                        let edit_emp = edit_emp_clone.clone();
                                                                        move |_| edit_empire((*edit_emp).clone())
                                                                    }
                                                                    class="btn btn-small btn-secondary"
                                                                >
                                                                    "Edit"
                                                                </button>
                                                                <button 
                                                                    on:click=move |_| delete_empire_action(delete_id)
                                                                    class="btn btn-small btn-danger"
                                                                >
                                                                    "Delete"
                                                                </button>
                                                            </Show>
                                                        </td>
                                                    </tr>
                                                }
                                            }
                                        />
                                    </tbody>
                                </table>
                            }.into_view()
                        }}
                    </div>
                }.into_view()
            }}
        </div>
    }
}

#[component]
pub fn UsersPage() -> impl IntoView {
    let (users, set_users) = create_signal(Vec::<ApiUser>::new());
    let (show_form, set_show_form) = create_signal(false);
    let (editing_user, set_editing_user) = create_signal(None::<ApiUser>);
    let (form_data, set_form_data) = create_signal(None::<UpsertUser>);
    let (cancel_form, set_cancel_form) = create_signal(false);
    let (error, set_error) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(false);
    let (auth_state, set_auth_state) = create_signal(is_authenticated());

    // Update auth state reactively
    create_effect(move |_| {
        set_auth_state.set(is_authenticated());
    });

    // Load users on mount
    create_effect(move |_| {
        spawn_local(async move {
            set_loading.set(true);
            match api::get_users().await {
                Ok(user_list) => set_users.set(user_list),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    // Handle form submission
    create_effect(move |_| {
        if let Some(data) = form_data.get() {
            spawn_local(async move {
                set_loading.set(true);
                let result = if let Some(user) = editing_user.get() {
                    api::update_user(user.id, data).await
                } else {
                    api::register(data.fullname, data.email, data.password, data.role).await
                };

                match result {
                    Ok(_) => {
                        // Reload users
                        match api::get_users().await {
                            Ok(user_list) => set_users.set(user_list),
                            Err(e) => set_error.set(Some(e)),
                        }
                        set_show_form.set(false);
                        set_editing_user.set(None);
                    },
                    Err(e) => set_error.set(Some(e)),
                }
                set_loading.set(false);
            });
            set_form_data.set(None);
        }
    });

    // Handle form cancel
    create_effect(move |_| {
        if cancel_form.get() {
            set_show_form.set(false);
            set_editing_user.set(None);
            set_cancel_form.set(false);
        }
    });

    let add_user = move |_| {
        set_editing_user.set(None);
        set_show_form.set(true);
    };

    let edit_user = move |user: ApiUser| {
        set_editing_user.set(Some(user));
        set_show_form.set(true);
    };

    let delete_user_action = move |id: i32| {
        spawn_local(async move {
            set_loading.set(true);
            match api::delete_user(id).await {
                Ok(_) => {
                    match api::get_users().await {
                        Ok(user_list) => set_users.set(user_list),
                        Err(e) => set_error.set(Some(e)),
                    }
                },
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    };

    view! {
        <Navbar/>
        <div class="container">
            <h1>"Users"</h1>
            
            {move || error.get().map(|e| view! {
                <div class="error">{e}</div>
            })}

            {move || if show_form.get() {
                view! {
                    <UserForm
                        user=editing_user.get()
                        on_submit=set_form_data
                        on_cancel=set_cancel_form
                    />
                }.into_view()
            } else {
                view! {
                    <div class="actions">
                        {move || if auth_state.get() {
                            view! {
                                <button on:click=add_user class="btn btn-primary">"Add User"</button>
                            }.into_view()
                        } else {
                            view! {
                                <button disabled class="btn btn-secondary" title="Please log in to add users">"Add User"</button>
                            }.into_view()
                        }}
                    </div>

                    <div class="data-table">
                        {move || if loading.get() {
                            view! { <div class="loading">"Loading..."</div> }.into_view()
                        } else {
                            view! {
                                <table>
                                    <thead>
                                        <tr>
                                            <th>"ID"</th>
                                            <th>"Full Name"</th>
                                            <th>"Email"</th>
                                            <th>"Role"</th>
                                            <th>"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || users.get()
                                            key=|user| user.id
                                            children=move |user| {
                                                let edit_usr = std::rc::Rc::new(user.clone());
                                                let delete_id = user.id;
                                                let edit_usr_clone = edit_usr.clone();
                                                view! {
                                                    <tr>
                                                        <td>{user.id}</td>
                                                        <td>{user.fullname}</td>
                                                        <td>{user.email}</td>
                                                        <td>{user.role}</td>
                                                        <td class="actions">
                                                            <Show
                                                                when=move || auth_state.get()
                                                                fallback=move || view! {
                                                                    <button disabled class="btn btn-small btn-secondary" title="Please log in to edit">"Edit"</button>
                                                                    <button disabled class="btn btn-small btn-secondary" title="Please log in to delete">"Delete"</button>
                                                                }
                                                            >
                                                                <button 
                                                                    on:click={
                                                                        let edit_usr = edit_usr_clone.clone();
                                                                        move |_| edit_user((*edit_usr).clone())
                                                                    }
                                                                    class="btn btn-small btn-secondary"
                                                                >
                                                                    "Edit"
                                                                </button>
                                                                <button 
                                                                    on:click=move |_| delete_user_action(delete_id)
                                                                    class="btn btn-small btn-danger"
                                                                >
                                                                    "Delete"
                                                                </button>
                                                            </Show>
                                                        </td>
                                                    </tr>
                                                }
                                            }
                                        />
                                    </tbody>
                                </table>
                            }.into_view()
                        }}
                    </div>
                }.into_view()
            }}
        </div>
    }
}