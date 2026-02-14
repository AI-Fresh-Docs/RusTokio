// New Dashboard Page (using leptos-ui components)
use leptos::prelude::*;
use leptos_ui::{Card, CardHeader, CardContent, Badge, BadgeVariant};

use crate::providers::auth::use_auth;
use crate::providers::locale::translate;

#[component]
pub fn DashboardNew() -> impl IntoView {
    let auth = use_auth();

    // Mock stats (TODO: replace with GraphQL queries)
    let stats = vec![
        StatData {
            title: "Total Users",
            value: "2,543",
            change: "+12%",
            change_positive: true,
            icon: "👥",
        },
        StatData {
            title: "Total Posts",
            value: "1,284",
            change: "+8%",
            change_positive: true,
            icon: "📝",
        },
        StatData {
            title: "Total Orders",
            value: "892",
            change: "+23%",
            change_positive: true,
            icon: "📦",
        },
        StatData {
            title: "Revenue",
            value: "$45,231",
            change: "+15%",
            change_positive: true,
            icon: "💰",
        },
    ];

    // Mock recent activity
    let activities = vec![
        Activity {
            user: "John Doe",
            action: "created a new post",
            time: "2 minutes ago",
            icon: "📝",
        },
        Activity {
            user: "Jane Smith",
            action: "completed an order",
            time: "15 minutes ago",
            icon: "✅",
        },
        Activity {
            user: "Bob Wilson",
            action: "registered as a new user",
            time: "1 hour ago",
            icon: "👤",
        },
        Activity {
            user: "Alice Brown",
            action: "updated their profile",
            time: "2 hours ago",
            icon: "✏️",
        },
    ];

    view! {
        <div class="space-y-6">
            // Welcome Header
            <div class="mb-8">
                <h1 class="text-3xl font-bold text-gray-900">
                    {move || {
                        let name = auth.user
                            .get()
                            .and_then(|u| u.name.clone())
                            .unwrap_or_else(|| "User".to_string());
                        format!("Welcome back, {}!", name)
                    }}
                </h1>
                <p class="mt-2 text-gray-600">
                    "Here's what's happening with your platform today."
                </p>
            </div>

            // Stats Grid
            <div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4">
                {stats.into_iter().map(|stat| {
                    view! { <StatCard stat=stat /> }
                }).collect_view()}
            </div>

            // Main Content Grid
            <div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
                // Recent Activity (2 columns)
                <div class="lg:col-span-2">
                    <Card>
                        <CardHeader class="border-b border-gray-200">
                            <h3 class="text-lg font-semibold text-gray-900">
                                "Recent Activity"
                            </h3>
                        </CardHeader>
                        <CardContent>
                            <div class="space-y-4">
                                {activities.into_iter().map(|activity| {
                                    view! { <ActivityItem activity=activity /> }
                                }).collect_view()}
                            </div>
                        </CardContent>
                    </Card>
                </div>

                // Quick Actions (1 column)
                <div>
                    <Card>
                        <CardHeader class="border-b border-gray-200">
                            <h3 class="text-lg font-semibold text-gray-900">
                                "Quick Actions"
                            </h3>
                        </CardHeader>
                        <CardContent>
                            <div class="space-y-3">
                                <QuickActionLink href="/users" icon="👥">
                                    "Manage Users"
                                </QuickActionLink>
                                <QuickActionLink href="/posts" icon="📝">
                                    "Create Post"
                                </QuickActionLink>
                                <QuickActionLink href="/products" icon="🛍️">
                                    "Add Product"
                                </QuickActionLink>
                                <QuickActionLink href="/settings" icon="⚙️">
                                    "System Settings"
                                </QuickActionLink>
                            </div>
                        </CardContent>
                    </Card>
                </div>
            </div>
        </div>
    }
}

// ============================================================================
// StatCard Component
// ============================================================================

#[derive(Clone)]
struct StatData {
    title: &'static str,
    value: &'static str,
    change: &'static str,
    change_positive: bool,
    icon: &'static str,
}

#[component]
fn StatCard(stat: StatData) -> impl IntoView {
    let change_color = if stat.change_positive {
        "text-green-600"
    } else {
        "text-red-600"
    };

    view! {
        <Card class="hover:shadow-lg transition-shadow">
            <CardContent class="p-6">
                <div class="flex items-center justify-between">
                    <div class="flex-1">
                        <p class="text-sm font-medium text-gray-600">
                            {stat.title}
                        </p>
                        <p class="mt-2 text-3xl font-bold text-gray-900">
                            {stat.value}
                        </p>
                        <p class=format!("mt-2 text-sm font-medium {}", change_color)>
                            {stat.change}
                            " from last month"
                        </p>
                    </div>
                    <div class="ml-4 text-4xl">
                        {stat.icon}
                    </div>
                </div>
            </CardContent>
        </Card>
    }
}

// ============================================================================
// ActivityItem Component
// ============================================================================

#[derive(Clone)]
struct Activity {
    user: &'static str,
    action: &'static str,
    time: &'static str,
    icon: &'static str,
}

#[component]
fn ActivityItem(activity: Activity) -> impl IntoView {
    view! {
        <div class="flex items-start gap-4">
            <div class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full bg-gray-100 text-xl">
                {activity.icon}
            </div>
            <div class="flex-1">
                <p class="text-sm text-gray-900">
                    <span class="font-semibold">{activity.user}</span>
                    " "
                    {activity.action}
                </p>
                <p class="mt-1 text-xs text-gray-500">
                    {activity.time}
                </p>
            </div>
        </div>
    }
}

// ============================================================================
// QuickActionLink Component
// ============================================================================

#[component]
fn QuickActionLink(
    href: &'static str,
    icon: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <a
            href=href
            class="flex items-center gap-3 rounded-lg border border-gray-200 px-4 py-3 text-sm font-medium text-gray-700 transition-colors hover:bg-gray-50 hover:border-gray-300"
        >
            <span class="text-xl">{icon}</span>
            <span>{children()}</span>
        </a>
    }
}
