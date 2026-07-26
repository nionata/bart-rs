use std::{rc::Rc, time::Duration};

use bart::{BartClient, Etd, StationEtd};
use leptos::prelude::*;
use leptos::task;

#[component]
pub fn App() -> impl IntoView {
    let client = Rc::new(BartClient::new());

    let (etds, set_etds) = signal(Vec::<StationEtd>::new());
    let (elapsed, set_elapsed) = signal(0u32);
    let (error, set_error) = signal(Option::<String>::None);

    let fetch = {
        move || {
            let client = client.clone();
            task::spawn_local(async move {
                match client.estimates("GLEN").await {
                    Ok(data) => {
                        set_etds.set(data);
                        set_elapsed.set(0);
                        set_error.set(None);
                    }
                    Err(e) => set_error.set(Some(e.to_string())),
                }
            });
        }
    };

    fetch();

    set_interval(fetch, Duration::from_secs(60));
    set_interval(
        move || set_elapsed.update(|s| *s += 1),
        Duration::from_secs(1),
    );

    view! {
        <div class="card">
            {move || error.get().map(|e| view! { <p class="error">"Error: " {e}</p> })}
            <p class="elapsed">{move || {
                let s = elapsed.get();
                if s == 0 { "Updated just now".to_string() } else { format!("Updated {s}s ago") }
            }}</p>
            {move || etds.get().into_iter().map(|stn| view! { <StationView stn/> }).collect_view()}
        </div>
    }
}

#[component]
fn StationView(stn: StationEtd) -> impl IntoView {
    let (north, south) = {
        let groups = stn.by_direction();
        let north: Vec<Etd> = groups.north.into_iter().cloned().collect();
        let south: Vec<Etd> = groups.south.into_iter().cloned().collect();
        (north, south)
    };

    view! {
        <div>
            <h3>{stn.name.clone()} " (" {stn.abbr.clone()} ")"</h3>
            {(!north.is_empty()).then(|| view! {
                <p><strong>"↑ North"</strong></p>
                <EtdTable etds=north/>
            })}
            {(!south.is_empty()).then(|| view! {
                <p><strong>"↓ South"</strong></p>
                <EtdTable etds=south/>
            })}
        </div>
    }
}

#[component]
fn EtdTable(etds: Vec<Etd>) -> impl IntoView {
    view! {
        <div class="etd-table">
            {etds.into_iter().map(|etd| {
                let hexcolor = etd.estimate.first().map(|e| e.hexcolor.clone()).unwrap_or_default();
                let parts: Vec<String> = etd.estimate.iter().map(|e| {
                    e.minutes.as_mins().map(|n| n.to_string()).unwrap_or_else(|| "Leaving".to_string())
                }).collect();
                let times = if parts.iter().any(|s| s == "Leaving") {
                    parts.join(", ")
                } else {
                    format!("{} min", parts.join(", "))
                };
                view! {
                    <div class="etd-row">
                        <svg class="etd-dot" width="12" height="12">
                            <circle cx="6" cy="6" r="6" fill=hexcolor/>
                        </svg>
                        <span class="destination">{etd.destination}</span>
                        <span class="times">{times}</span>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
