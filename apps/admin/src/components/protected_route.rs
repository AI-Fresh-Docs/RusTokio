use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_router::hooks::use_navigate;

use crate::providers::auth::use_auth;

#[component]
pub fn ProtectedRoute() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    Effect::new(move |_| {
        if auth.token.get().is_none() {
            navigate("/login", Default::default());
        }
    });

    view! {
        <Show
            when=move || auth.token.get().is_some()
            fallback=|| ()
        >
            <Outlet />
        </Show>
    }
}
