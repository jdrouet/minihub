use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

pub mod api;
mod components;
mod pages;
pub mod sse;

use components::{Nav, ToastContainer};
use pages::{
    Areas, AutomationDetail, Automations, DeviceDetail, Devices, Entities, EntityDetail, Events,
    Home, NotFound,
};

/// Root application component.
#[component]
pub fn App() -> impl IntoView {
    view! {
        <ToastContainer>
            <Router>
                <Nav/>
                <main>
                    <Routes fallback=|| view! { <NotFound/> }>
                        <Route path=path!("/") view=Home/>
                        <Route path=path!("devices") view=Devices/>
                        <Route path=path!("devices/:id") view=DeviceDetail/>
                        <Route path=path!("entities") view=Entities/>
                        <Route path=path!("entities/:id") view=EntityDetail/>
                        <Route path=path!("areas") view=Areas/>
                        <Route path=path!("events") view=Events/>
                        <Route path=path!("automations") view=Automations/>
                        <Route path=path!("automations/:id") view=AutomationDetail/>
                    </Routes>
                </main>
            </Router>
        </ToastContainer>
    }
}
