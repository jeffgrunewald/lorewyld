//! Shared breadcrumb trail rendered at the top of every main page.
//! Static segments are server-rendered; entity-name leaves render an
//! ellipsis placeholder carrying an `id` the page script fills in
//! once its fetch resolves.

use leptos::prelude::*;

/// One segment in a breadcrumb trail.
pub struct Crumb {
    label: String,
    href: Option<String>,
    id: Option<&'static str>,
}

impl Crumb {
    /// Intermediate segment linking back up the hierarchy.
    pub fn link(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: Some(href.into()),
            id: None,
        }
    }

    /// Static current-page segment.
    pub fn here(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: None,
            id: None,
        }
    }

    /// Current-page segment whose label the page script fills in after
    /// fetching the entity it names.
    pub fn slot(id: &'static str) -> Self {
        Self {
            label: "…".to_string(),
            href: None,
            id: Some(id),
        }
    }

    /// Linked segment whose label the page script fills in.
    pub fn link_slot(id: &'static str, href: impl Into<String>) -> Self {
        Self {
            label: "…".to_string(),
            href: Some(href.into()),
            id: Some(id),
        }
    }
}

#[component]
pub fn Breadcrumbs(trail: Vec<Crumb>) -> impl IntoView {
    let last = trail.len().saturating_sub(1);
    view! {
        <nav class="lw-breadcrumbs" aria-label="Breadcrumb">
            <ol>
                {trail
                    .into_iter()
                    .enumerate()
                    .map(|(index, crumb)| match crumb.href {
                        Some(href) => view! {
                            <li>
                                <a href=href id=crumb.id>{crumb.label}</a>
                            </li>
                        }
                            .into_any(),
                        None => view! {
                            <li>
                                <span
                                    aria-current=(index == last).then_some("page")
                                    id=crumb.id
                                >
                                    {crumb.label}
                                </span>
                            </li>
                        }
                            .into_any(),
                    })
                    .collect_view()}
            </ol>
        </nav>
    }
}
