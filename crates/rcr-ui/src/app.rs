use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::components::{Router, Routes, Route};
use leptos_router::StaticSegment;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Run Crab Run" />
        <Router>
            <nav class="navbar">
                <a href="/" class="nav-brand">"🦀 Run Crab Run"</a>
                <div class="nav-links">
                    <a href="/jobs">"Jobs"</a>
                    <a href="/runs">"Runs"</a>
                </div>
            </nav>
            <main class="container">
                <Routes fallback=|| view! { <NotFound /> }>
                    <Route path=StaticSegment("") view=HomePage />
                    <Route path=StaticSegment("jobs") view=JobsPage />
                    <Route path=StaticSegment("runs") view=RunsPage />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="dashboard">
            <h1>"🦀 Run Crab Run"</h1>
            <p>"Job runner dashboard"</p>
            <p>"Use the API or the Jobs/Runs pages to manage your jobs."</p>
        </div>
    }
}

#[component]
fn JobsPage() -> impl IntoView {
    view! {
        <div class="page">
            <h1>"Jobs"</h1>
            <p>"Job management — use the API to create and manage jobs."</p>
        </div>
    }
}

#[component]
fn RunsPage() -> impl IntoView {
    view! {
        <div class="page">
            <h1>"Runs"</h1>
            <p>"Run history — use the API to view runs."</p>
        </div>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="not-found">
            <h1>"404"</h1>
            <p>"Page not found"</p>
            <a href="/">"Go home"</a>
        </div>
    }
}