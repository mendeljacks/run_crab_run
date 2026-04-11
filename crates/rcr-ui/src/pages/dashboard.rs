use leptos::prelude::*;
use leptos::task::spawn_local;
use rcr_core::models::{Job, RunStatus};
use rcr_core::models::run::RunSummary;

use crate::api;
use crate::util::*;

#[component]
pub fn DashboardPage() -> impl IntoView {
    let (jobs, set_jobs) = signal(Vec::<Job>::new());
    let (runs, set_runs) = signal(Vec::<RunSummary>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (delete_id, set_delete_id) = signal(None::<String>);

    // Initial fetch
    spawn_local(async move {
        match api::fetch_jobs().await {
            Ok(j) => set_jobs.set(j),
            Err(e) => set_error.set(Some(e)),
        }
        match api::fetch_runs(20).await {
            Ok(r) => set_runs.set(r),
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
                                    {move || runs.get().iter().filter(|r| r.status == RunStatus::Running).count()}
                                </div>
                                <div class="stat-label">"Running"</div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-num stat-success">
                                    {move || runs.get().iter().filter(|r| r.status == RunStatus::Success).count()}
                                </div>
                                <div class="stat-label">"Success"</div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-num stat-danger">
                                    {move || runs.get().iter().filter(|r| r.status == RunStatus::Failed).count()}
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
                                            let _id_href = format!("/api/jobs/{}/trigger", job.id);

                                            view! {
                                                <div class="card">
                                                    <div class="job-card-head">
                                                        <strong>{job.name.clone()}</strong>
                                                        <span class=badge_class>{badge_text}</span>
                                                    </div>
                                                    <div class="mono">{job.command.clone()}</div>
                                                    <div class="meta">{schedule}</div>
                                                    <div class="tag-row">
                                                        {job.tags.iter().map(|tag| view! {
                                                            <span class="badge badge-neutral">{tag.clone()}</span>
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                    <div class="card-actions">
                                                        <button class="btn btn-primary btn-sm" on:click=move |_| {
                                                            let id = id_trigger.clone();
                                                            spawn_local(async move {
                                                                if let Err(e) = api::trigger_job(&id).await {
                                                                    set_error.set(Some(e));
                                                                } else if let Ok(r) = api::fetch_runs(20).await {
                                                                    set_runs.set(r);
                                                                }
                                                            });
                                                        }>"▶ Run"</button>
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
                            let r = runs.get();
                            if r.is_empty() {
                                view! { <div class="alert alert-neutral">"No runs yet."</div> }.into_any()
                            } else {
                                view! {
                                    <div class="table-wrap">
                                        <table class="data-table">
                                            <thead><tr><th>"ID"</th><th>"Job"</th><th>"Status"</th><th>"Trigger"</th><th>"Duration"</th><th>"Started"</th></tr></thead>
                                            <tbody>
                                                {r.into_iter().map(|run| {
                                                    let (sc, st) = status_badge(&run.status);
                                                    let rid = run.id.clone();
                                                    let _href = format!("/runs/{}", run.id);
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