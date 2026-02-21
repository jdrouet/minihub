//! Area table component for displaying a list of areas.

use leptos::prelude::*;
use minihub_domain::area::Area;

/// A table displaying a list of areas.
#[component]
pub fn AreaTable(
    /// The list of areas to display.
    areas: Vec<Area>,
) -> impl IntoView {
    if areas.is_empty() {
        view! {
            <p>"No areas found."</p>
        }
        .into_any()
    } else {
        view! {
            <table>
                <thead>
                    <tr>
                        <th>"Name"</th>
                        <th>"Parent"</th>
                    </tr>
                </thead>
                <tbody>
                    {areas.iter().map(|area| {
                        view! {
                            <AreaRow area=area.clone()/>
                        }
                    }).collect::<Vec<_>>()}
                </tbody>
            </table>
        }
        .into_any()
    }
}

/// A single row in the area table.
#[component]
fn AreaRow(
    /// The area to display.
    area: Area,
) -> impl IntoView {
    let name = area.name;
    let parent = area.parent_id.map_or("—".to_string(), |p| p.to_string());

    view! {
        <tr>
            <td>{name}</td>
            <td>{parent}</td>
        </tr>
    }
}
