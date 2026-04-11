use leptos::prelude::*;
use leptos::task::spawn_local;
use rcr_core::models::Job;

use crate::api;
use crate::util::*;

#[component]
pub fn JobsPage() -> impl IntoView {
    let (jobs, set_jobs) = signal(Vec::<Job>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (delete_id, set_delete_id) = signal(None::<String>);

    spawn_local(async move {
        match api::fetch_jobs().await {
            Ok(j) => set_jobs.set(j),
            Err(e) => set_error.set(Some(e)),
        }
        set_loading.set(false);
    });

    view! {
        <div class="view">
            <div class="view-header">
                <h1>"Jobs"</h1>
                <a href="/jobs/new" class="btn btn-primary">"+ New Job"</a>
            </div>

            {move || error.get().map(|e| view! { <div class="alert alert-danger">{e}</div> })}

            {move || if loading.get() {
                view! { <div class="center-msg"><div class="spinner spinner-lg"></div></div> }.into_any()
            } else {
                let j = jobs.get();
                if j.is_empty() {
                    view! { <div class="alert alert-neutral">"No jobs yet. Create one!"</div> }.into_any()
                } else {
                    view! {
                        <div class="table-wrap">
                            <table class="data-table">
                                <thead>
                                    <tr>
                                        <th>"Name"</th>
                                        <th>"Command"</th>
                                        <th>"Schedule"</th>
                                        <th>"Status"</th>
                                        <th>"Container"</th>
                                        <th>"Notify"</th>
                                        <th>"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {j.into_iter().map(|job| {
                                        let (badge_class, badge_text) = enabled_badge(job.enabled);
                                        let id_trigger = job.id.clone();
                                        let id_delete = job.id.clone();
                                        let id_edit = job.id.clone();
                                        let id_runs = job.id.clone();
                                        let schedule = job.schedule.clone().unwrap_or_else(|| "—".to_string());

                                        view! {
                                            <tr>
                                                <td>
                                                    <a href={format!("/runs?job_id={}", id_runs)} class="job-link">
                                                        <strong>{job.name.clone()}</strong>
                                                    </a>
                                                </td>
                                                <td><code>{job.command.clone()}</code></td>
                                                <td>{schedule}</td>
                                                <td><span class=badge_class>{badge_text}</span></td>
                                                <td>
                                                    {if job.containerized {
                                                        "🐳 Yes".to_string()
                                                    } else {
                                                        "—".to_string()
                                                    }}
                                                </td>
                                                <td>
                                                    {if job.notify {
                                                        format!("📧 {}", job.notify_email.as_deref().unwrap_or("—"))
                                                    } else {
                                                        "—".to_string()
                                                    }}
                                                </td>
                                                <td class="action-cell">
                                                    <button class="btn btn-primary btn-sm" on:click=move |_| {
                                                        let id = id_trigger.clone();
                                                        set_error.set(None);
                                                        spawn_local(async move {
                                                            if let Err(e) = api::trigger_job(&id).await {
                                                                set_error.set(Some(e));
                                                            }
                                                        });
                                                    }>"▶ Run"</button>
                                                    <a href={format!("/jobs/{}/edit", id_edit)} class="btn btn-default btn-sm">"✎ Edit"</a>
                                                    <button class="btn btn-danger btn-sm" on:click=move |_| {
                                                        set_delete_id.set(Some(id_delete.clone()));
                                                    }>"✕ Delete"</button>
                                                </td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        </div>
                    }.into_any()
                }
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