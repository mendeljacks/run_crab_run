use leptos::prelude::*;
use leptos::task::spawn_local;
use rcr_core::models::run::RunSummary;

use crate::api;
use crate::util::*;

#[component]
pub fn RunsPage() -> impl IntoView {
    let (runs, set_runs) = signal(Vec::<RunSummary>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    spawn_local(async move {
        match api::fetch_runs(100).await {
            Ok(r) => set_runs.set(r),
            Err(e) => set_error.set(Some(e)),
        }
        set_loading.set(false);
    });

    view! {
        <div class="view">
            <div class="view-header">
                <h1>"Runs"</h1>
            </div>

            {move || error.get().map(|e| view! { <div class="alert alert-danger">{e}</div> })}

            {move || if loading.get() {
                view! { <div class="center-msg"><div class="spinner spinner-lg"></div></div> }.into_any()
            } else {
                let r = runs.get();
                if r.is_empty() {
                    view! { <div class="alert alert-neutral">"No runs yet."</div> }.into_any()
                } else {
                    view! {
                        <div class="table-wrap">
                            <table class="data-table">
                                <thead>
                                    <tr>
                                        <th>"ID"</th>
                                        <th>"Job"</th>
                                        <th>"Status"</th>
                                        <th>"Trigger"</th>
                                        <th>"Duration"</th>
                                        <th>"Started"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {r.into_iter().map(|run| {
                                        let (sc, st) = status_badge(&run.status);
                                        let rid = run.id.clone();
                                        view! {
                                            <tr class="clickable" on:click=move |_| {
                                                let _ = web_sys::window().unwrap().location().set_href(&format!("/runs/{}", rid));
                                            }>
                                                <td><code>{short_id(&run.id)}</code></td>
                                                <td>{run.job_name.clone()}</td>
                                                <td><span class=sc>{st}</span></td>
                                                <td>{format_trigger(&run.trigger)}</td>
                                                <td>{format_duration(run.duration_ms)}</td>
                                                <td>{format_time_ago(&run.started_at)}</td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}