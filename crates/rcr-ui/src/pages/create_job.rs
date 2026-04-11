use leptos::prelude::*;
use leptos::task::spawn_local;
use rcr_core::models::job::CreateJob;

use crate::api;

#[component]
pub fn CreateJobPage() -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (command, set_command) = signal(String::new());
    let (schedule, set_schedule) = signal(String::new());
    let (timeout_secs, set_timeout_secs) = signal(String::new());
    let (tags, set_tags) = signal(String::new());
    let (notify_on, set_notify_on) = signal(String::from("never"));
    let (notify_email, set_notify_email) = signal(String::new());
    let (submitting, set_submitting) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (success, set_success) = signal(false);

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

        let create = CreateJob {
            name: n,
            command: c,
            schedule: if schedule.get().is_empty() { None } else { Some(schedule.get()) },
            enabled: None,
            timeout_secs: timeout_secs.get().parse().ok(),
            max_concurrent: None,
            env_vars: None,
            webhook_secret: None,
            tags: if tags.get().is_empty() {
                None
            } else {
                Some(tags.get().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
            },
            notify_on: if notify_on.get() == "never" { None } else { Some(notify_on.get()) },
            notify_email: if notify_email.get().is_empty() { None } else { Some(notify_email.get()) },
        };

        set_submitting.set(true);
        spawn_local(async move {
            match api::create_job(&create).await {
                Ok(_job) => {
                    set_success.set(true);
                    set_submitting.set(false);
                    // Navigate to jobs page after a short delay
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

            <div class="card" style="max-width: 700px;">
                <form on:submit=on_submit>
                    <div class="form-grid">
                        <div class="form-group">
                            <label>"Name *"</label>
                            <input class="form-input" type="text"
                                placeholder="e.g., typecheck"
                                on:input=move |ev| set_name.set(event_target_value(&ev)) />
                        </div>

                        <div class="form-group">
                            <label>"Command *"</label>
                            <input class="form-input" type="text"
                                placeholder="e.g., cargo check"
                                on:input=move |ev| set_command.set(event_target_value(&ev)) />
                        </div>

                        <div class="form-group">
                            <label>"Schedule (RRULE)"</label>
                            <input class="form-input" type="text"
                                placeholder="FREQ=DAILY;BYHOUR=9"
                                on:input=move |ev| set_schedule.set(event_target_value(&ev)) />
                            <div class="form-hint">"RRULE format, or leave empty for manual/webhook-only"</div>
                        </div>

                        <div class="form-group-half">
                            <label>"Timeout (seconds)"</label>
                            <input class="form-input" type="number"
                                placeholder="300"
                                on:input=move |ev| set_timeout_secs.set(event_target_value(&ev)) />
                        </div>

                        <div class="form-group-half">
                            <label>"Tags (comma separated)"</label>
                            <input class="form-input" type="text"
                                placeholder="ci, rust"
                                on:input=move |ev| set_tags.set(event_target_value(&ev)) />
                        </div>

                        <div class="form-group-half">
                            <label>"Notify on"</label>
                            <select class="form-select"
                                on:change=move |ev| set_notify_on.set(event_target_value(&ev))>
                                <option value="never">"Never"</option>
                                <option value="failure">"Failure only"</option>
                                <option value="always">"Always"</option>
                            </select>
                        </div>

                        <div class="form-group-half">
                            <label>"Notify email"</label>
                            <input class="form-input" type="email"
                                placeholder="dev@example.com"
                                on:input=move |ev| set_notify_email.set(event_target_value(&ev)) />
                        </div>

                        <div class="form-actions">
                            <button class="btn btn-primary" type="submit"
                                disabled=move || submitting.get()>
                                {move || if submitting.get() { "Creating..." } else { "Create Job" }}
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