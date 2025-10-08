use crate::components::{NavBar, TopBar};
use crate::pages::{
    RolePage, StudentEditProfilePage, StudentHomePage, StudentLoginPage, StudentProfilePage,
    StudentStatisticsPage, Support, TermsAndConditions,
};
use crate::routes::{ ClassQrFullscreenPage, ClassQrPage, ClassesPage, EditClass, EditModule, Error, HomePage,
    Login, NewClass, NewModule, Profile, Register, Statistics, Timetable,
};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::StaticSegment;
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_location,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    crate::user_context::init_user_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/clock-it.css?v=20251007001"/>
        <Title text="Clock-It"/>
        <Router>
            <AppShell/>
        </Router>
    }
}

#[component]
fn AppShell() -> impl IntoView {
    let location = use_location();
    let show_topbar = {
        let location = location.clone();
        Signal::derive(move || {
            let path = location.pathname.get();
            path.starts_with("/home")
                || path.starts_with("/timetable")
                || path.starts_with("/statistics")
                || path.starts_with("/about")
                || path.starts_with("/classes")
                || path.starts_with("/modules")
                || path.starts_with("/lecturer/profile")
        })
    };
    let show_sidebar = {
        let location = location.clone();
        Signal::derive(move || {
            let path = location.pathname.get();
            path.starts_with("/home")
                || path.starts_with("/timetable")
                || path.starts_with("/statistics")
                || path.starts_with("/about")
                || path.starts_with("/classes")
                || path.starts_with("/modules")
        })
    };
    let show_footer = Signal::derive(move || show_sidebar.get());
    let shell_class = Signal::derive(move || {
        if show_sidebar.get() {
            "app-shell".to_string()
        } else {
            "app-shell no-sidebar".to_string()
        }
    });
    view! {
    <div class=move || shell_class.get()>
            <Show when=move || show_topbar.get()>
                <TopBar/>
            </Show>
            <Show when=move || show_sidebar.get()>
                <NavBar/>
            </Show>
            <main class="content">
                <Routes fallback=|| view! { <Error/> }>
                    <Route path=StaticSegment("") view=Login/>
                    <Route path=StaticSegment("register") view=Register/>
                    <Route path=StaticSegment("terms") view=TermsAndConditions/>
                    <Route path=StaticSegment("home") view=HomePage/>
                    <Route path=StaticSegment("classes") view=ClassesPage/>
                    <Route path=(StaticSegment("classes"), StaticSegment("new")) view=NewClass/>
                    <Route path=(StaticSegment("modules"), StaticSegment("new")) view=NewModule/>
                    <Route path=(StaticSegment("modules"), StaticSegment("edit")) view=EditModule/>
                    <Route path=StaticSegment("timetable") view=Timetable/>
                    <Route path=StaticSegment("statistics") view=Statistics/>
                    <Route path=(StaticSegment("lecturer"), StaticSegment("profile")) view=Profile/>
                    <Route path=(StaticSegment("classes"), StaticSegment("edit")) view=EditClass/>
                    <Route path=(StaticSegment("classes"), StaticSegment("qr")) view=ClassQrPage/>
                    <Route path=(StaticSegment("classes"), StaticSegment("qr"), StaticSegment("large")) view=ClassQrFullscreenPage/>
                    // add student pages
                    <Route path=(StaticSegment("student"), StaticSegment("home")) view=StudentHomePage/>
                    <Route path=(StaticSegment("student"), StaticSegment("login")) view=StudentLoginPage/>
                    <Route path=(StaticSegment("student"), StaticSegment("role")) view=RolePage/>
                    <Route path=(StaticSegment("student"), StaticSegment("profile")) view=StudentProfilePage/>
                    <Route path=(StaticSegment("student"), StaticSegment("profile"), StaticSegment("edit")) view=StudentEditProfilePage/>
                    <Route path=(StaticSegment("student"), StaticSegment("statistics")) view=StudentStatisticsPage/>
                    <Route path=(StaticSegment("student"), StaticSegment("support")) view=Support/>
                </Routes>
            </main>
            <Show when=move || show_footer.get()>
                <footer class="footer">
                    <small>"Built with Leptos"</small>
                </footer>
            </Show>
        </div>
    }
}
