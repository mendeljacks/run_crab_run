use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use rcr_core::models::Run;

use crate::api;
use crate::util::*;

#[component]
pub fn RunDetailPage() -> impl IntoView {
    let params = use_params_map();
    let run_id = params.with(|p| p.get("id").unwrap_or_default());

    let (run, set_run) = signal(None::<Run>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    // Fetch run details
    let id_for_fetch = run_id.clone();
    spawn_local(async move {
        match api::fetch_run(&id_for_fetch).await {
            Ok(r) => set_run.set(Some(r)),
            Err(e) => set_error.set(Some(e)),
        }
        set_loading.set(false);
    });

    view! {
        <div class="view">
            {move || if loading.get() {
                view! { <div class="center-msg"><div class="spinner spinner-lg"></div></div> }.into_any()
            } else if let Some(e) = error.get() {
                view! { <div class="alert alert-danger">{e}</div> }.into_any()
            } else if let Some(r) = run.get() {
                let (status_class, status_text) = status_badge(&r.status);
                let job_id_link = r.job_id.clone();
                let job_name_display = r.job_name.as_deref().unwrap_or(&r.job_id).to_string();
                let is_running = r.status == rcr_core::models::RunStatus::Running;
                let run_id_for_cancel = r.id.clone();
                let command_display = r.command.as_deref().unwrap_or("—").to_string();
                view! {
                    <>
                        <div class="view-header">
                            <h1>{format!("Run {}", short_id(&r.id))}</h1>
                            <span class=status_class>{status_text}</span>
                        </div>

                        <div class="detail-grid">
                            <div class="detail-fact">
                                <div class="detail-label">"Job"</div>
                                <div class="detail-value">
                                    <a href={format!("/runs?job_id={}", job_id_link)} class="job-link">{job_name_display}</a>
                                </div>
                            </div>
                            <div class="detail-fact">
                                <div class="detail-label">"Command"</div>
                                <div class="detail-value">
                                    <code class="command-display">{command_display}</code>
                                </div>
                            </div>
                            <div class="detail-fact">
                                <div class="detail-label">"Trigger"</div>
                                <div class="detail-value">{format_trigger(&r.trigger)}</div>
                            </div>
                            <div class="detail-fact">
                                <div class="detail-label">"Exit Code"</div>
                                <div class="detail-value">{r.exit_code.map(|c| c.to_string()).unwrap_or_else(|| "—".to_string())}</div>
                            </div>
                            <div class="detail-fact">
                                <div class="detail-label">"Duration"</div>
                                <div class="detail-value">{format_duration(r.duration_ms)}</div>
                            </div>
                            <div class="detail-fact">
                                <div class="detail-label">"CPU Peak"</div>
                                <div class="detail-value">{format_percent(r.cpu_pct)}</div>
                            </div>
                            <div class="detail-fact">
                                <div class="detail-label">"Memory Peak"</div>
                                <div class="detail-value">{format_memory(r.mem_kb)}</div>
                            </div>
                            <div class="detail-fact">
                                <div class="detail-label">"Started"</div>
                                <div class="detail-value">{format_datetime(&r.started_at)}</div>
                            </div>
                            <div class="detail-fact">
                                <div class="detail-label">"Finished"</div>
                                <div class="detail-value">{r.finished_at.map(|f| format_datetime(&f)).unwrap_or_else(|| "—".to_string())}</div>
                            </div>
                        </div>

                        {r.error_message.as_ref().map(|err| view! {
                            <div class="alert alert-danger">
                                <strong>"Error: "</strong>{err.clone()}
                            </div>
                        })}

                        {r.stdout.as_ref().map(|out| view! {
                            <>
                                <h3>"Stdout"</h3>
                                <pre class="output-box">{out.clone()}</pre>
                            </>
                        })}

                        {r.stderr.as_ref().map(|err| view! {
                            <>
                                <h3>"Stderr"</h3>
                                <pre class="output-box output-stderr">{err.clone()}</pre>
                            </>
                        })}

                        <div style="margin-top: 1.5rem; display: flex; gap: 0.5rem;">
                            <a href="/runs" class="btn btn-default">"← Back to Runs"</a>
                            {if is_running {
                                view! {
                                    <button class="btn btn-danger" on:click=move |_| {
                                        let id = run_id_for_cancel.clone();
                                        spawn_local(async move {
                                            let _ = api::cancel_run(&id).await;
                                        });
                                    }>"✕ Cancel Run"</button>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }}
                        </div>
                    </>
                }.into_any()
            } else {
                view! { <div class="alert alert-neutral">"Run not found."</div> }.into_any()
            }}
        </div>
    }
}