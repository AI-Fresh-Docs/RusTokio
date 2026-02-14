// New Users List Page (using leptos-ui components)
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_ui::{Card, CardHeader, CardContent, Badge, BadgeVariant, Button, ButtonVariant, Input};

#[component]
pub fn UsersNew() -> impl IntoView {
    // Mock data (TODO: replace with GraphQL query)
    let users = vec![
        UserData {
            id: "1",
            name: "John Doe",
            email: "john@example.com",
            role: "admin",
            status: "active",
            created_at: "2024-01-15",
        },
        UserData {
            id: "2",
            name: "Jane Smith",
            email: "jane@example.com",
            role: "editor",
            status: "active",
            created_at: "2024-01-20",
        },
        UserData {
            id: "3",
            name: "Bob Wilson",
            email: "bob@example.com",
            role: "user",
            status: "inactive",
            created_at: "2024-02-01",
        },
        UserData {
            id: "4",
            name: "Alice Brown",
            email: "alice@example.com",
            role: "editor",
            status: "active",
            created_at: "2024-02-10",
        },
    ];

    let (search_query, set_search_query) = signal(String::new());

    view! {
        <div class="space-y-6">
            // Page Header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Users"</h1>
                    <p class="mt-1 text-sm text-gray-600">
                        "Manage your platform users"
                    </p>
                </div>
                <Button variant=ButtonVariant::Primary>
                    "➕ Add User"
                </Button>
            </div>

            // Filters & Search
            <Card>
                <CardContent class="p-4">
                    <div class="flex items-center gap-4">
                        <div class="flex-1">
                            <Input
                                placeholder=Some("Search users...")
                                value=Some(search_query.read_only())
                                on_input=Some(Box::new(move |ev| {
                                    let value = leptos::ev::event_target_value(&ev);
                                    set_search_query.set(value);
                                }))
                            />
                        </div>
                        <select class="rounded-md border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:ring-blue-500">
                            <option>"All Roles"</option>
                            <option>"Admin"</option>
                            <option>"Editor"</option>
                            <option>"User"</option>
                        </select>
                        <select class="rounded-md border border-gray-300 px-3 py-2 text-sm focus:border-blue-500 focus:ring-blue-500">
                            <option>"All Status"</option>
                            <option>"Active"</option>
                            <option>"Inactive"</option>
                        </select>
                    </div>
                </CardContent>
            </Card>

            // Users Table
            <Card>
                <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200">
                        <thead class="bg-gray-50">
                            <tr>
                                <th class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                                    "User"
                                </th>
                                <th class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                                    "Role"
                                </th>
                                <th class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                                    "Status"
                                </th>
                                <th class="px-6 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">
                                    "Created"
                                </th>
                                <th class="px-6 py-3 text-right text-xs font-medium uppercase tracking-wider text-gray-500">
                                    "Actions"
                                </th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-200 bg-white">
                            {users.into_iter().map(|user| {
                                view! { <UserRow user=user /> }
                            }).collect_view()}
                        </tbody>
                    </table>
                </div>

                // Pagination
                <div class="border-t border-gray-200 bg-gray-50 px-6 py-4">
                    <div class="flex items-center justify-between">
                        <div class="text-sm text-gray-700">
                            "Showing "
                            <span class="font-medium">"1"</span>
                            " to "
                            <span class="font-medium">"4"</span>
                            " of "
                            <span class="font-medium">"4"</span>
                            " results"
                        </div>
                        <div class="flex gap-2">
                            <Button
                                variant=ButtonVariant::Outline
                                disabled=true
                            >
                                "Previous"
                            </Button>
                            <Button
                                variant=ButtonVariant::Outline
                                disabled=true
                            >
                                "Next"
                            </Button>
                        </div>
                    </div>
                </div>
            </Card>
        </div>
    }
}

// ============================================================================
// UserRow Component
// ============================================================================

#[derive(Clone)]
struct UserData {
    id: &'static str,
    name: &'static str,
    email: &'static str,
    role: &'static str,
    status: &'static str,
    created_at: &'static str,
}

#[component]
fn UserRow(user: UserData) -> impl IntoView {
    let role_badge = match user.role {
        "admin" => BadgeVariant::Primary,
        "editor" => BadgeVariant::Warning,
        _ => BadgeVariant::Default,
    };

    let status_badge = match user.status {
        "active" => BadgeVariant::Success,
        "inactive" => BadgeVariant::Danger,
        _ => BadgeVariant::Default,
    };

    view! {
        <tr class="hover:bg-gray-50">
            <td class="px-6 py-4 whitespace-nowrap">
                <div class="flex items-center">
                    <div class="h-10 w-10 flex-shrink-0">
                        <div class="h-10 w-10 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
                            <span class="text-white text-sm font-semibold">
                                {user.name.chars().next().unwrap_or('U').to_string()}
                            </span>
                        </div>
                    </div>
                    <div class="ml-4">
                        <div class="text-sm font-medium text-gray-900">
                            {user.name}
                        </div>
                        <div class="text-sm text-gray-500">
                            {user.email}
                        </div>
                    </div>
                </div>
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
                <Badge variant=role_badge>
                    {user.role}
                </Badge>
            </td>
            <td class="px-6 py-4 whitespace-nowrap">
                <Badge variant=status_badge>
                    {user.status}
                </Badge>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                {user.created_at}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                <div class="flex items-center justify-end gap-2">
                    <A
                        href=format!("/users/{}", user.id)
                        class="text-blue-600 hover:text-blue-900"
                    >
                        "View"
                    </A>
                    <button class="text-gray-600 hover:text-gray-900">
                        "Edit"
                    </button>
                    <button class="text-red-600 hover:text-red-900">
                        "Delete"
                    </button>
                </div>
            </td>
        </tr>
    }
}
