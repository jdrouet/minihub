use leptos::prelude::*;

use crate::api;
use crate::components::AutomationTable;

/// Automations page displaying all automations with enable/disable toggle.
#[component]
pub fn Automations() -> impl IntoView {
    let (reload_trigger, set_reload_trigger) = signal(0);

    let automations = LocalResource::new(move || {
        reload_trigger.track();
        api::fetch_automations()
    });

    let handle_update = move || {
        set_reload_trigger.update(|v| *v += 1);
    };

    view! {
        <div>
            <h1>"Automations"</h1>
            <Suspense fallback=move || view! { <p>"Loading automations…"</p> }>
                {move || {
                    automations.read().as_ref().map(|result| match result {
                        Ok(automations_list) => view! {
                            <AutomationTable
                                automations=automations_list.clone()
                                on_update=handle_update
                            />
                        }.into_any(),
                        Err(err) => view! {
                            <p class="error">{"Failed to load automations: "} {err.to_string()}</p>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}
