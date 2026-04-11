use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::StaticSegment;
use leptos_router::ParamSegment;

use crate::pages::{dashboard, jobs, create_job, edit_job, runs, run_detail};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Run Crab Run" />
        <Router>
            <nav class="navbar">
                <a href="/" class="nav-brand">"🦀 Run Crab Run"</a>
                <div class="nav-links">
                    <a href="/" class="nav-link">"Dashboard"</a>
                    <a href="/jobs" class="nav-link">"Jobs"</a>
                    <a href="/runs" class="nav-link">"Runs"</a>
                </div>
            </nav>
            <main class="container">
                <Routes fallback=|| view! { <NotFound /> }>
                    <Route path=StaticSegment("") view=dashboard::DashboardPage />
                    <Route path=StaticSegment("jobs") view=jobs::JobsPage />
                    <Route path=(StaticSegment("jobs"), StaticSegment("new")) view=create_job::CreateJobPage />
                    <Route path=(StaticSegment("jobs"), ParamSegment("id"), StaticSegment("edit")) view=edit_job::EditJobPage />
                    <Route path=StaticSegment("runs") view=runs::RunsPage />
                    <Route path=(StaticSegment("runs"), ParamSegment("id")) view=run_detail::RunDetailPage />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="not-found">
            <h1>"404"</h1>
            <p>"Page not found"</p>
            <a href="/" class="btn btn-primary">"Go home"</a>
        </div>
    }
}