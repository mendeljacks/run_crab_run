use leptos::prelude::*;
use leptos::task::spawn_local;
use rcr_core::models::Run;

use crate::api;
use crate::util::*;

#[component]
pub fn RunDetailPage() -> impl IntoView {
    // Extract run ID from the URL path: /runs/{id}
    let path = web_sys::window().unwrap().location().pathname().unwrap_or_default();
    let run_id = path.rsplit('/').next().unwrap_or_default().to_string();

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
                view! {
                    <>
                        <div class="view-header">
                            <h1>{format!("Run {}", short_id(&r.id))}</h1>
                            <span class=status_class>{status_text}</span>
                        </div>

                        <div class="detail-grid">
                            <div class="detail-fact">
                                <div class="detail-label">"Job"</div>
                                <div class="detail-value">{r.job_id.clone()}</div>
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

                        <div style="margin-top: 1.5rem">
                            <a href="/runs" class="btn btn-default">"← Back to Runs"</a>
                        </div>
                    </>
                }.into_any()
            } else {
                view! { <div class="alert alert-neutral">"Run not found."</div> }.into_any()
            }}
        </div>
    }
}