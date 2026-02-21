use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

mod components;
mod pages;

use components::Nav;
use pages::{
    Areas, AutomationDetail, Automations, Devices, Entities, EntityDetail, Events, Home, NotFound,
};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Nav/>
            <main>
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=path!("/") view=Home/>
                    <Route path=path!("/devices") view=Devices/>
                    <Route path=path!("/entities") view=Entities/>
                    <Route path=path!("/entities/:id") view=EntityDetail/>
                    <Route path=path!("/areas") view=Areas/>
                    <Route path=path!("/events") view=Events/>
                    <Route path=path!("/automations") view=Automations/>
                    <Route path=path!("/automations/:id") view=AutomationDetail/>
                </Routes>
            </main>
        </Router>
    }
}
