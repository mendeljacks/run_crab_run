use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use rcr_core::models::job::UpdateJob;

use crate::api;

#[component]
pub fn EditJobPage() -> impl IntoView {
    let params = use_params_map();
    let job_id = params.with(|p| p.get("id").unwrap_or_default());

    let (loading, set_loading) = signal(true);
    let (load_error, set_load_error) = signal(None::<String>);

    let (name, set_name) = signal(String::new());
    let (command, set_command) = signal(String::new());
    let (schedule, set_schedule) = signal(String::new());
    let (containerized, set_containerized) = signal(false);
    let (container_image, set_container_image) = signal("alpine:latest".to_string());
    let (submitting, set_submitting) = signal(false);
    let (submit_error, set_submit_error) = signal(None::<String>);
    let (success, set_success) = signal(false);

    let id_for_fetch = job_id.clone();
    spawn_local(async move {
        match api::fetch_job(&id_for_fetch).await {
            Ok(j) => {
                set_name.set(j.name.clone());
                set_command.set(j.command.clone());
                set_schedule.set(j.schedule.clone().unwrap_or_default());
                set_containerized.set(j.containerized);
                set_container_image.set(j.container_image.clone().unwrap_or_else(|| "alpine:latest".to_string()));
                set_loading.set(false);
            }
            Err(e) => {
                set_load_error.set(Some(e));
                set_loading.set(false);
            }
        }
    });

    let command_preview = move || {
        let cmd = command.get();
        let is_containerized = containerized.get();
        let img = container_image.get();
        if cmd.is_empty() {
            "Enter a command above to see the preview…".to_string()
        } else if is_containerized {
            format!("docker run --rm {} bash -c '{}'", img, cmd.replace('\'', "'\\''"))
        } else {
            format!("bash -c '{}'", cmd.replace('\'', "'\\''"))
        }
    };

    let job_id_stored = StoredValue::new(job_id.clone());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_submit_error.set(None);
        set_success.set(false);

        let is_containerized = containerized.get();
        let sched = schedule.get();
        let ci = container_image.get();
        let update = UpdateJob {
            name: Some(name.get()),
            command: Some(command.get()),
            schedule: if sched.is_empty() { None } else { Some(sched) },
            enabled: None,
            max_concurrent: None,
            env_vars: None,
            containerized: Some(is_containerized),
            container_image: if is_containerized { Some(ci) } else { None },
        };

        let id = job_id_stored.get_value();
        set_submitting.set(true);
        spawn_local(async move {
            match api::update_job(&id, &update).await {
                Ok(_updated) => {
                    set_success.set(true);
                    set_submitting.set(false);
                }
                Err(e) => {
                    set_submit_error.set(Some(e));
                    set_submitting.set(false);
                }
            }
        });
    };

    view! {
        <div class="view">
            <div class="view-header">
                <h1>"Edit Job"</h1>
                <a href="/jobs" class="btn btn-default">"← Back to Jobs"</a>
            </div>

            {move || if loading.get() {
                view! { <div class="center-msg"><div class="spinner spinner-lg"></div></div> }.into_any()
            } else if let Some(e) = load_error.get() {
                view! { <div class="alert alert-danger">{e}</div> }.into_any()
            } else {
                view! {
                    <div class="card" style="max-width: 800px;">
                        <form on:submit=on_submit>
                            <div class="form-grid">
                                <div class="form-group">
                                    <label>"Name *"</label>
                                    <input class="form-input" type="text"
                                        value=name.get()
                                        on:input=move |ev| set_name.set(event_target_value(&ev)) />
                                </div>

                                <div class="form-group">
                                    <label>"Command *"</label>
                                    <input class="form-input" type="text"
                                        value=command.get()
                                        on:input=move |ev| set_command.set(event_target_value(&ev)) />
                                    <div class="form-hint">"The shell command to execute (runs inside bash -c)"</div>
                                </div>

                                <div class="form-group">
                                    <label>"Command Preview"</label>
                                    <pre class="command-preview">{command_preview}</pre>
                                </div>

                                <div class="form-group">
                                    <label>"Schedule (RRULE)"</label>
                                    <input class="form-input" type="text"
                                        value=schedule.get()
                                        placeholder="FREQ=DAILY;BYHOUR=9"
                                        on:input=move |ev| set_schedule.set(event_target_value(&ev)) />
                                    <div class="form-hint">"RRULE format, or leave empty for manual-only"</div>
                                </div>

                                <div class="form-group-half">
                                    <label class="checkbox-label">
                                        <input type="checkbox"
                                            checked=containerized.get()
                                            on:change=move |ev| set_containerized.set(event_target_checked(&ev)) />
                                        " Run Containerized (Docker)"
                                    </label>
                                </div>

                                {move || if containerized.get() {
                                    view! {
                                        <div class="form-group">
                                            <label>"Container Image"</label>
                                            <input class="form-input" type="text"
                                                value=container_image.get()
                                                placeholder="alpine:latest"
                                                on:input=move |ev| set_container_image.set(event_target_value(&ev)) />
                                            <div class="form-hint">"Docker image to use."</div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <div></div> }.into_any()
                                }}

                                <div class="form-actions">
                                    <button class="btn btn-primary" type="submit"
                                        disabled=move || submitting.get()>
                                        {move || if submitting.get() { "Saving…" } else { "Save Changes" }}
                                    </button>
                                    <a href="/jobs" class="btn btn-default">"Cancel"</a>
                                </div>

                                {move || submit_error.get().map(|e| view! {
                                    <div class="alert alert-danger">{e}</div>
                                })}

                                {move || if success.get() {
                                    Some(view! { <div class="alert alert-success">"Job updated!"</div> })
                                } else {
                                    None
                                }}
                            </div>
                        </form>
                    </div>
                }.into_any()
            }}
        </div>
    }
}