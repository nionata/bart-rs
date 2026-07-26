use std::sync::Arc;

use bart::{BartClient, Etd, StationEtd};
use gloo_timers::callback::Interval;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

fn main() {
    mount_to_body(|| view! { <App/> });
}

#[component]
fn App() -> impl IntoView {
    let client = Arc::new(BartClient::new());

    let (etds, set_etds) = signal(Vec::<StationEtd>::new());
    let (elapsed, set_elapsed) = signal(0u32);
    let (error, set_error) = signal(Option::<String>::None);

    let fetch = {
        let client = client.clone();
        move || {
            let client = client.clone();
            spawn_local(async move {
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

    // Timers must outlive this function. WASM is single-threaded and the
    // browser cancels all timers when the page closes, so leaking is safe.
    std::mem::forget(Interval::new(60_000, fetch));
    std::mem::forget(Interval::new(1_000, move || {
        set_elapsed.update(|s| *s += 1)
    }));

    view! {
        <style>"body { margin: 0; background: #1e1e1e; color: #fff; display: flex; justify-content: center; padding: 0 1rem; box-sizing: border-box; } @media (max-width: 600px) { .card { margin-top: 2rem !important; font-size: 1.1rem; } }"</style>
        <div class="card" style="font-family: monospace; padding: 1.5rem; width: 100%; max-width: 480px; margin-top: 10rem; border: 1px solid #333; border-radius: 8px; box-shadow: 0 0 18px rgba(255,200,50,0.06), 0 0 4px rgba(255,200,50,0.04); box-sizing: border-box;">

            {move || error.get().map(|e| view! { <p style="color: red;">"Error: " {e}</p> })}
            <p style="font-size: 0.8em; opacity: 0.45; margin-top: 0;">{move || {
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
        <div style="margin-bottom: 0.5rem;">
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
                    <div style="display: flex; align-items: center; padding: 0.35rem 0; gap: 0.75rem;">
                        <svg width="12" height="12" style="flex-shrink: 0;">
                            <circle cx="6" cy="6" r="6" fill=hexcolor/>
                        </svg>
                        <span style="flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">{etd.destination}</span>
                        <span style="white-space: nowrap; flex-shrink: 0;">{times}</span>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}
