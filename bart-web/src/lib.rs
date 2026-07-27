use std::{rc::Rc, time::Duration};

use bart::{BartClient, Etd, Station, StationEtd};
use leptos::ev;
use leptos::prelude::*;
use leptos::task;

fn non_empty_location_hash() -> Option<String> {
    location_hash().filter(|hash| !hash.is_empty())
}

#[component]
pub fn App() -> impl IntoView {
    let (station, set_station) = signal(non_empty_location_hash());
    let (stations, set_stations) = signal(Vec::<Station>::new());

    let client = Rc::new(BartClient::new());
    task::spawn_local(async move {
        if let Ok(mut list) = client.stations().await {
            list.sort_by(|a, b| a.name.cmp(&b.name));
            set_stations.set(list);
        }
    });

    let handle = window_event_listener(ev::hashchange, move |_| {
        set_station.set(non_empty_location_hash());
    });
    on_cleanup(move || handle.remove());

    view! {
        <img class="map-bg" src="assets/map.svg" aria-hidden="true"/>
        {move || {
            if stations.get().is_empty() {
                return ().into_any();
            }
            match station.get() {
                None => view! { <StationPicker set_station stations/> }.into_any(),
                Some(abbr) => {
                    match stations.get().into_iter().find(|s| s.abbr == abbr) {
                        Some(station) => view! { <DepartureBoard station/> }.into_any(),
                        None => view! {
                            <div class="card">
                                <p class="error">"Unknown station: " {abbr}</p>
                            </div>
                        }.into_any(),
                    }
                }
            }
        }}
    }
}

#[component]
fn StationPicker(
    set_station: WriteSignal<Option<String>>,
    stations: ReadSignal<Vec<Station>>,
) -> impl IntoView {
    view! {
        <div class="card picker">
            <select
                on:change=move |ev| {
                    let abbr = event_target_value(&ev);
                    if !abbr.is_empty() {
                        set_station.set(Some(abbr.clone()));
                        let _ = window().location().set_hash(&abbr);
                    }
                }
            >
                <option value="" disabled=true selected=true>"Select a station…"</option>
                {move || stations.get().into_iter().map(|s| view! {
                    <option value=s.abbr>{s.name}</option>
                }).collect_view()}
            </select>
        </div>
    }
}

#[component]
fn DepartureBoard(station: Station) -> impl IntoView {
    let client = Rc::new(BartClient::new());
    let (etds, set_etds) = signal(Vec::<StationEtd>::new());
    let (elapsed, set_elapsed) = signal(0u32);
    let (error, set_error) = signal(Option::<String>::None);

    let station_name = format!("{} ({})", station.name, station.abbr);
    let abbr = station.abbr;

    let fetch = move || {
        let client = client.clone();
        let abbr = abbr.clone();
        task::spawn_local(async move {
            match client.estimates(&abbr).await {
                Ok(data) => {
                    set_etds.set(data);
                    set_elapsed.set(0);
                    set_error.set(None);
                }
                Err(e) => set_error.set(Some(e.to_string())),
            }
        });
    };

    fetch();

    set_interval(fetch, Duration::from_secs(60));
    set_interval(
        move || set_elapsed.update(|s| *s += 1),
        Duration::from_secs(1),
    );

    view! {
        <div class="card">
            <div class="station-header">
                <h3>{station_name}</h3>
                <button class="change-station" on:click=move |_| {
                    let _ = window().location().set_hash("");
                }>"×"</button>
            </div>
            {move || {
                if let Some(e) = error.get() {
                    return view! { <p class="error">"Error: " {e}</p> }.into_any();
                }
                let data = etds.get();
                if data.is_empty() {
                    return view! { <p class="loading">"Loading…"</p> }.into_any();
                }
                view! {
                    <>
                        <p class="elapsed">{move || {
                            let s = elapsed.get();
                            if s == 0 { "Updated just now".to_string() } else { format!("Updated {s}s ago") }
                        }}</p>
                        {if data.iter().all(|s| s.etd.is_empty()) {
                            view! { <p class="loading">"No departures available."</p> }.into_any()
                        } else {
                            data.into_iter().map(|stn| view! { <StationView stn/> }).collect_view().into_any()
                        }}
                    </>
                }.into_any()
            }}
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
