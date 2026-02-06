use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn PageHeader(
    #[prop(into)] title: String,
    #[prop(optional, into)] subtitle: Option<String>,
    #[prop(optional, into)] eyebrow: Option<String>,
    #[prop(optional)] actions: Option<Children>,
    #[prop(optional)] breadcrumbs: Option<Vec<(String, String)>>, // (Label, Href)
) -> impl IntoView {
    view! {
        <header class="mb-8 flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
            <div>
                {move || eyebrow.clone().map(|text| view! {
                    <span class="inline-flex items-center rounded-full bg-slate-200 px-3 py-1 text-xs font-semibold text-slate-600 mb-2">
                        {text}
                    </span>
                })}

                <h1 class="text-2xl font-semibold text-slate-900">{title.clone()}</h1>

                {move || subtitle.clone().map(|text| view! {
                    <p class="mt-2 text-sm text-slate-500">{text}</p>
                })}

                {move || breadcrumbs.clone().map(|crumbs| view! {
                    <div class="mt-4 flex items-center gap-2 text-sm text-slate-500">
                        {crumbs.into_iter().enumerate().map(|(index, (label, href))| {
                            view! {
                                <Show when=move || index > 0>
                                    <span class="text-slate-300">"/"</span>
                                </Show>
                                <A href=href class="hover:text-slate-900 transition-colors">
                                    {label}
                                </A>
                            }
                        }).collect_view()}
                    </div>
                })}
            </div>

            {move || actions.as_ref().map(|children| view! {
                <div class="flex flex-wrap items-center gap-3">
                    {children()}
                </div>
            })}
        </header>
    }
}
