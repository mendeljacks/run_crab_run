use leptos::prelude::*;
use leptos::task::spawn_local;
use rcr_core::models::job::CreateJob;

use crate::api;

#[component]
pub fn CreateJobPage() -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (command, set_command) = signal(String::new());
    let (schedule, set_schedule) = signal(String::new());
    let (containerized, set_containerized) = signal(false);
    let (container_image, set_container_image) = signal("alpine:latest".to_string());
    let (submitting, set_submitting) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (success, set_success) = signal(false);

    let command_preview = move || {
        let cmd = command.get();
        let is_containerized = containerized.get();
        let image = container_image.get();
        if cmd.is_empty() {
            "Enter a command above to see the preview…".to_string()
        } else if is_containerized {
            format!("docker run --rm {} bash -c '{}'", image, cmd.replace('\'', "'\\''"))
        } else {
            format!("bash -c '{}'", cmd.replace('\'', "'\\''"))
        }
    };

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        set_success.set(false);

        let n = name.get();
        let c = command.get();
        if n.is_empty() || c.is_empty() {
            set_error.set(Some("Name and command are required.".to_string()));
            return;
        }

        let is_containerized = containerized.get();
        let create = CreateJob {
            name: n,
            command: c,
            schedule: if schedule.get().is_empty() { None } else { Some(schedule.get()) },
            enabled: None,
            max_concurrent: None,
            env_vars: None,
            containerized: Some(is_containerized),
            container_image: if is_containerized { Some(container_image.get()) } else { None },
        };

        set_submitting.set(true);
        spawn_local(async move {
            match api::create_job(&create).await {
                Ok(_job) => {
                    set_success.set(true);
                    set_submitting.set(false);
                    gloo_timers::callback::Timeout::new(800, || {
                        let _ = web_sys::window().unwrap().location().set_href("/jobs");
                    }).forget();
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_submitting.set(false);
                }
            }
        });
    };

    view! {
        <div class="view">
            <div class="view-header">
                <h1>"Create Job"</h1>
            </div>

            <div class="card" style="max-width: 800px;">
                <form on:submit=on_submit>
                    <div class="form-grid">
                        <div class="form-group">
                            <label>"Name *"</label>
                            <input class="form-input" type="text"
                                placeholder="e.g., deploy-prod"
                                on:input=move |ev| set_name.set(event_target_value(&ev)) />
                        </div>

                        <div class="form-group">
                            <label>"Command *"</label>
                            <input class="form-input" type="text"
                                placeholder="e.g., ./deploy.sh"
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
                                    <div class="form-hint">"Docker image to use. The command will run inside a container."</div>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }}

                        <div class="form-actions">
                            <button class="btn btn-primary" type="submit"
                                disabled=move || submitting.get()>
                                {move || if submitting.get() { "Creating…" } else { "Create Job" }}
                            </button>
                            <a href="/jobs" class="btn btn-default">"Cancel"</a>
                        </div>

                        {move || error.get().map(|e| view! {
                            <div class="alert alert-danger">{e}</div>
                        })}

                        {move || if success.get() {
                            Some(view! { <div class="alert alert-success">"Job created!"</div> })
                        } else {
                            None
                        }}
                    </div>
                </form>
            </div>
        </div>
    }
}