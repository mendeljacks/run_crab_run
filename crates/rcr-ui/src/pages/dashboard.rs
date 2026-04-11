use leptos::prelude::*;
use leptos::task::spawn_local;
use rcr_core::models::Job;
use rcr_core::models::run::RunStatus;
use rcr_core::models::run::RunsFilter;

use crate::api;
use crate::util::*;

#[component]
pub fn DashboardPage() -> impl IntoView {
    let (jobs, set_jobs) = signal(Vec::<Job>::new());
    let (runs_data, set_runs_data) = signal(None::<(Vec<rcr_core::models::run::RunSummary>, i64)>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (delete_id, set_delete_id) = signal(None::<String>);

    // Initial fetch
    spawn_local(async move {
        match api::fetch_jobs().await {
            Ok(j) => set_jobs.set(j),
            Err(e) => set_error.set(Some(e)),
        }
        let filter = RunsFilter {
            job_id: None,
            status: None,
            search: None,
            sort_by: Some("started_at".to_string()),
            sort_order: Some("desc".to_string()),
            limit: Some(20),
            offset: Some(0),
        };
        match api::fetch_runs(&filter).await {
            Ok(resp) => set_runs_data.set(Some((resp.runs, resp.total))),
            Err(e) => set_error.set(Some(e)),
        }
        set_loading.set(false);
    });

    view! {
        <div class="view">
            <div class="view-header">
                <h1>"Dashboard"</h1>
            </div>

            {move || error.get().map(|e| view! { <div class="alert alert-danger">{e}</div> })}

            {move || if loading.get() {
                view! {
                    <div class="center-msg">
                        <div class="spinner spinner-lg"></div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <>
                        <div class="stat-row">
                            <div class="stat-card">
                                <div class="stat-num">{move || jobs.get().len()}</div>
                                <div class="stat-label">"Jobs"</div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-num stat-primary">
                                    {move || runs_data.get().map(|(runs, _)| runs.iter().filter(|r| r.status == RunStatus::Running).count()).unwrap_or(0)}
                                </div>
                                <div class="stat-label">"Running"</div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-num stat-success">
                                    {move || runs_data.get().map(|(runs, _)| runs.iter().filter(|r| r.status == RunStatus::Success).count()).unwrap_or(0)}
                                </div>
                                <div class="stat-label">"Success"</div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-num stat-danger">
                                    {move || runs_data.get().map(|(runs, _)| runs.iter().filter(|r| r.status == RunStatus::Failed).count()).unwrap_or(0)}
                                </div>
                                <div class="stat-label">"Failed"</div>
                            </div>
                        </div>

                        <h2>"Jobs"</h2>
                        {move || {
                            let j = jobs.get();
                            if j.is_empty() {
                                view! { <div class="alert alert-neutral">"No jobs yet. Create one!"</div> }.into_any()
                            } else {
                                view! {
                                    <div class="job-grid">
                                        {j.iter().map(|job| {
                                            let (badge_class, badge_text) = enabled_badge(job.enabled);
                                            let schedule = job.schedule.clone().map(|s| format!("📅 {}", s)).unwrap_or_else(|| "⚡ Manual / Webhook".to_string());
                                            let id_trigger = job.id.clone();
                                            let id_delete = job.id.clone();
                                            let id_edit = job.id.clone();
                                            let id_runs = job.id.clone();

                                            view! {
                                                <div class="card">
                                                    <div class="job-card-head">
                                                        <a href={format!("/runs?job_id={}", id_runs)} class="job-link">
                                                            <strong>{job.name.clone()}</strong>
                                                        </a>
                                                        <span class=badge_class>{badge_text}</span>
                                                    </div>
                                                    <div class="mono">{job.command.clone()}</div>
                                                    <div class="meta">{schedule}</div>
                                                    <div class="meta">
                                                        {if job.containerized { "🐳 Containerized" } else { "" }}
                                                        " "
                                                        {if job.notify { "📧 Notify" } else { "" }}
                                                    </div>
                                                    <div class="card-actions">
                                                        <button class="btn btn-primary btn-sm" on:click=move |_| {
                                                            let id = id_trigger.clone();
                                                            spawn_local(async move {
                                                                if let Err(e) = api::trigger_job(&id).await {
                                                                    set_error.set(Some(e));
                                                                } else {
                                                                    let filter = RunsFilter {
                                                                        job_id: None,
                                                                        status: None,
                                                                        search: None,
                                                                        sort_by: Some("started_at".to_string()),
                                                                        sort_order: Some("desc".to_string()),
                                                                        limit: Some(20),
                                                                        offset: Some(0),
                                                                    };
                                                                    if let Ok(resp) = api::fetch_runs(&filter).await {
                                                                        set_runs_data.set(Some((resp.runs, resp.total)));
                                                                    }
                                                                }
                                                            });
                                                        }>"▶ Run"</button>
                                                        <a href={format!("/jobs/{}/edit", id_edit)} class="btn btn-default btn-sm">"✎ Edit"</a>
                                                        <button class="btn btn-danger btn-sm" on:click=move |_| {
                                                            set_delete_id.set(Some(id_delete.clone()));
                                                        }>"✕ Delete"</button>
                                                    </div>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            }
                        }}

                        <h2 style="margin-top:2rem">"Recent Runs"</h2>
                        {move || {
                            if let Some((runs, _)) = runs_data.get() {
                                if runs.is_empty() {
                                    view! { <div class="alert alert-neutral">"No runs yet."</div> }.into_any()
                                } else {
                                    view! {
                                        <div class="table-wrap">
                                            <table class="data-table">
                                                <thead><tr><th>"ID"</th><th>"Job"</th><th>"Status"</th><th>"Trigger"</th><th>"Duration"</th><th>"Started"</th></tr></thead>
                                                <tbody>
                                                    {runs.into_iter().map(|run| {
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
                            } else {
                                view! { <div class="alert alert-neutral">"No runs yet."</div> }.into_any()
                            }
                        }}
                    </>
                }.into_any()
            }}

            {move || delete_id.get().map(|id| {
                let confirm_id = id.clone();
                view! {
                    <div class="modal-overlay" on:click=move |ev| {
                        if ev.target() == ev.current_target() {
                            set_delete_id.set(None);
                        }
                    }>
                        <div class="modal">
                            <h3>"Delete Job"</h3>
                            <p>"Are you sure you want to delete this job?"</p>
                            <div class="modal-actions">
                                <button class="btn btn-default" on:click=move |_| set_delete_id.set(None)>"Cancel"</button>
                                <button class="btn btn-danger" on:click=move |_| {
                                    let id = confirm_id.clone();
                                    spawn_local(async move {
                                        let _ = api::delete_job(&id).await;
                                        set_delete_id.set(None);
                                        if let Ok(j) = api::fetch_jobs().await { set_jobs.set(j); }
                                    });
                                }>"Delete"</button>
                            </div>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}