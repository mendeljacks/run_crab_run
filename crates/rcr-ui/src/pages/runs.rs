use leptos::prelude::*;
use leptos::task::spawn_local;
use rcr_core::models::run::{RunStatus, RunsFilter};

use crate::api;
use crate::util::*;

const PAGE_SIZE: i64 = 25;

#[component]
pub fn RunsPage() -> impl IntoView {
    let (runs_data, set_runs_data) = signal(None::<(Vec<rcr_core::models::run::RunSummary>, i64)>);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);
    let (page, set_page) = signal(0i64);
    let (search, set_search) = signal(String::new());
    let (status_filter, set_status_filter) = signal(String::new());
    let (sort_by, _set_sort_by) = signal(String::from("started_at"));
    let (sort_order, set_sort_order) = signal(String::from("desc"));

    let search_params = web_sys::window()
        .and_then(|w| w.location().search().ok())
        .unwrap_or_default();
    let initial_job_id = extract_query_param(&search_params, "job_id");
    let (job_filter, set_job_filter) = signal(initial_job_id);

    let fetch_runs = move || {
        let filter = RunsFilter {
            job_id: if job_filter.get().is_empty() { None } else { Some(job_filter.get()) },
            status: if status_filter.get().is_empty() { None } else { parse_status(&status_filter.get()) },
            search: if search.get().is_empty() { None } else { Some(search.get()) },
            sort_by: if sort_by.get().is_empty() { None } else { Some(sort_by.get()) },
            sort_order: if sort_order.get().is_empty() { None } else { Some(sort_order.get()) },
            limit: Some(PAGE_SIZE),
            offset: Some(page.get() * PAGE_SIZE),
        };
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match api::fetch_runs(&filter).await {
                Ok(resp) => {
                    set_runs_data.set(Some((resp.runs, resp.total)));
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    // Initial fetch
    spawn_local(async move {
        let filter = RunsFilter {
            job_id: if job_filter.get().is_empty() { None } else { Some(job_filter.get()) },
            status: None,
            search: None,
            sort_by: Some("started_at".to_string()),
            sort_order: Some("desc".to_string()),
            limit: Some(PAGE_SIZE),
            offset: Some(0),
        };
        match api::fetch_runs(&filter).await {
            Ok(resp) => {
                set_runs_data.set(Some((resp.runs, resp.total)));
                set_loading.set(false);
            }
            Err(e) => {
                set_error.set(Some(e));
                set_loading.set(false);
            }
        }
    });

    let total_pages = move || {
        runs_data.get()
            .map(|(_, total)| ((total as f64) / (PAGE_SIZE as f64)).ceil() as i64)
            .unwrap_or(1)
            .max(1)
    };

    view! {
        <div class="view">
            <div class="view-header">
                <h1>"Runs"</h1>
            </div>

            <div class="filters-bar">
                <input class="form-input filter-input"
                    type="text"
                    placeholder="Search by job name…"
                    value=search.get()
                    on:input=move |ev| {
                        set_search.set(event_target_value(&ev));
                        set_page.set(0);
                    }
                />
                <select class="form-select filter-input"
                    on:change=move |ev| {
                        set_status_filter.set(event_target_value(&ev));
                        set_page.set(0);
                    }
                >
                    <option value="">"All statuses"</option>
                    <option value="running">"Running"</option>
                    <option value="success">"Success"</option>
                    <option value="failed">"Failed"</option>
                    <option value="timeout">"Timeout"</option>
                </select>
                <select class="form-select filter-input"
                    on:change=move |ev| {
                        set_sort_order.set(event_target_value(&ev));
                    }
                >
                    <option value="desc">"Newest first"</option>
                    <option value="asc">"Oldest first"</option>
                </select>
                <button class="btn btn-primary btn-sm" on:click=move |_| {
                    fetch_runs();
                }>"Apply"</button>
                {move || if !job_filter.get().is_empty() {
                    view! {
                        <button class="btn btn-default btn-sm" on:click=move |_| {
                            set_job_filter.set(String::new());
                            set_page.set(0);
                            fetch_runs();
                        }>"Clear Job Filter"</button>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }}
            </div>

            {move || error.get().map(|e| view! { <div class="alert alert-danger">{e}</div> })}

            {move || if loading.get() {
                view! { <div class="center-msg"><div class="spinner spinner-lg"></div></div> }.into_any()
            } else if let Some((runs, total)) = runs_data.get() {
                if runs.is_empty() {
                    view! { <div class="alert alert-neutral">"No runs found."</div> }.into_any()
                } else {
                    let tp = total_pages();
                    view! {
                        <>
                            <div class="results-info">
                                {format!("{} runs (showing page {} of {})", total, page.get() + 1, tp)}
                            </div>
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
                            <div class="pagination">
                                <button class="btn btn-default btn-sm"
                                    disabled=move || page.get() == 0
                                    on:click=move |_| {
                                        set_page.update(|p| *p = (*p).saturating_sub(1));
                                        fetch_runs();
                                    }
                                >"← Prev"</button>
                                <span class="page-info">
                                    {format!("Page {} of {}", page.get() + 1, tp)}
                                </span>
                                <button class="btn btn-default btn-sm"
                                    disabled=move || page.get() >= tp - 1
                                    on:click=move |_| {
                                        set_page.update(|p| *p += 1);
                                        fetch_runs();
                                    }
                                >"Next →"</button>
                            </div>
                        </>
                    }.into_any()
                }
            } else {
                view! { <div class="alert alert-neutral">"No runs yet."</div> }.into_any()
            }}
        </div>
    }
}

fn extract_query_param(query: &str, key: &str) -> String {
    let search = query.trim_start_matches('?');
    for pair in search.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return urldecode(v);
            }
        }
    }
    String::new()
}

fn urldecode(s: &str) -> String {
    s.replace("%20", " ")
        .replace("%2F", "/")
        .replace("+", " ")
}

fn parse_status(s: &str) -> Option<RunStatus> {
    match s {
        "running" => Some(RunStatus::Running),
        "success" => Some(RunStatus::Success),
        "failed" => Some(RunStatus::Failed),
        "timeout" => Some(RunStatus::Timeout),
        _ => None,
    }
}