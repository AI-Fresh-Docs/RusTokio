// Register Page (новая версия с leptos-ui, leptos-forms, leptos-graphql)
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use leptos_forms::{use_form, Field, Validator};
use leptos_ui::{Button, ButtonVariant, Card, CardHeader, CardContent};
use leptos_auth::api;

use crate::providers::locale::translate;

#[component]
pub fn RegisterNew() -> impl IntoView {
    let navigate = use_navigate();
    
    // Form state через leptos-forms
    let form = use_form();
    
    // Register fields
    form.register("tenant");
    form.register("name");
    form.register("email");
    form.register("password");
    form.register("confirm_password");
    
    // Set validators
    form.set_validator("tenant", Validator::new().required());
    form.set_validator("name", Validator::new().required());
    form.set_validator("email", Validator::new().email().required());
    form.set_validator("password", Validator::new().min_length(8).required());
    
    // Custom validator for password confirmation
    form.set_validator(
        "confirm_password",
        Validator::new()
            .required()
            .custom(|value| {
                let password = form.get_value("password");
                if value != password {
                    Err("Passwords don't match".to_string())
                } else {
                    Ok(())
                }
            })
    );
    
    let (error, set_error) = signal(Option::<String>::None);
    let (is_loading, set_is_loading) = signal(false);
    
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        // Validate all fields
        if form.validate_all().is_err() {
            return;
        }
        
        let tenant = form.get_value("tenant");
        let name = form.get_value("name");
        let email = form.get_value("email");
        let password = form.get_value("password");
        
        set_error.set(None);
        set_is_loading.set(true);
        
        let navigate = navigate.clone();
        
        spawn_local(async move {
            match api::sign_up(email, password, Some(name), tenant).await {
                Ok((user, session)) => {
                    // Save to localStorage via leptos-auth storage
                    leptos_auth::storage::save_session(&session);
                    leptos_auth::storage::save_user(&user);
                    
                    // Navigate to dashboard
                    navigate("/dashboard", Default::default());
                }
                Err(err) => {
                    let message = match err {
                        leptos_auth::AuthError::Network => {
                            translate("errors.network").to_string()
                        }
                        _ => translate("errors.unknown").to_string(),
                    };
                    set_error.set(Some(message));
                    set_is_loading.set(false);
                }
            }
        });
    };
    
    view! {
        <section class="grid min-h-screen grid-cols-1 lg:grid-cols-[1.2fr_1fr]">
            // Hero section
            <aside class="flex flex-col justify-center gap-6 bg-[radial-gradient(circle_at_top_left,#1e3a8a,#0f172a)] p-12 text-white lg:p-16">
                <span class="inline-flex w-fit items-center rounded-full bg-white/10 px-3 py-1 text-xs font-semibold text-white/80">
                    "Create Account"
                </span>
                <h1 class="text-4xl font-semibold">
                    "Join RusToK Platform"
                </h1>
                <p class="text-lg text-white/80">
                    "Start managing your content and commerce in minutes"
                </p>
                <div>
                    <p class="text-sm font-semibold">
                        "What you'll get:"
                    </p>
                    <ul class="text-sm text-white/75 space-y-2 mt-2">
                        <li>"✓ Full admin access"</li>
                        <li>"✓ Real-time analytics"</li>
                        <li>"✓ Multi-tenant support"</li>
                        <li>"✓ GraphQL API"</li>
                    </ul>
                </div>
            </aside>
            
            // Form section
            <div class="flex flex-col justify-center gap-7 bg-slate-50 p-12 lg:p-20">
                <Card class="rounded-3xl shadow-[0_24px_60px_rgba(15,23,42,0.12)]">
                    <CardHeader>
                        <h2 class="text-2xl font-semibold">
                            "Create your account"
                        </h2>
                        <p class="text-slate-500">
                            "Fill in your details to get started"
                        </p>
                    </CardHeader>
                    
                    <CardContent>
                        <form on:submit=on_submit class="space-y-4">
                            // Error display
                            <Show when=move || error.get().is_some()>
                                <div class="rounded-xl bg-red-100 px-4 py-2 text-sm text-red-700">
                                    {move || error.get().unwrap_or_default()}
                                </div>
                            </Show>
                            
                            // Tenant field
                            <Field 
                                form=form 
                                name="tenant" 
                                label=Some("Tenant Slug")
                                placeholder=Some("my-company")
                            />
                            
                            // Name field
                            <Field 
                                form=form 
                                name="name" 
                                label=Some("Full Name")
                                placeholder=Some("John Doe")
                            />
                            
                            // Email field
                            <Field 
                                form=form 
                                name="email" 
                                label=Some("Email Address")
                                placeholder=Some("you@example.com")
                                r#type="email"
                            />
                            
                            // Password field
                            <Field 
                                form=form 
                                name="password" 
                                label=Some("Password")
                                placeholder=Some("••••••••")
                                r#type="password"
                            />
                            
                            // Confirm password field
                            <Field 
                                form=form 
                                name="confirm_password" 
                                label=Some("Confirm Password")
                                placeholder=Some("••••••••")
                                r#type="password"
                            />
                            
                            // Submit button
                            <Button 
                                variant=ButtonVariant::Primary
                                r#type=Some("submit")
                                class=Some("w-full")
                                loading=is_loading.get()
                                disabled=is_loading.get()
                            >
                                "Create Account"
                            </Button>
                            
                            // Links
                            <div class="text-center text-sm">
                                "Already have an account? "
                                <a class="text-blue-600 hover:underline" href="/login">
                                    "Sign in"
                                </a>
                            </div>
                        </form>
                    </CardContent>
                </Card>
                
                <p class="text-sm text-slate-500 text-center">
                    "By creating an account, you agree to our Terms of Service and Privacy Policy"
                </p>
            </div>
        </section>
    }
}
