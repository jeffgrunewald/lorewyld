use leptos::prelude::*;

/// Admin user-management page: paginated user table with delete /
/// admin-toggle actions and a manual add-user modal. Skeleton rendered
/// server-side; JS gates on the `lw-auth-ready` probe and populates
/// from `/api/admin/users`. The API itself is the real access gate —
/// this page just avoids showing admins-only chrome to other viewers.
#[component]
pub fn SettingsUsersPage() -> impl IntoView {
    view! {
        <section class="lw-settings">
            <p id="lw-settings-forbidden" class="lw-settings-forbidden" hidden=true>
                "You must be logged in as an administrator to view this page."
            </p>
            <div id="lw-settings-users-root" hidden=true>
                <header class="lw-settings-header">
                    <h1 class="lw-settings-title">"Users"</h1>
                    <button id="lw-users-add" class="lw-btn lw-btn-filled" type="button">
                        "Add user"
                    </button>
                </header>
                <table class="lw-table">
                    <thead>
                        <tr>
                            <th>"Username"</th>
                            <th>"Email"</th>
                            <th>"Admin"</th>
                            <th>"Actions"</th>
                        </tr>
                    </thead>
                    <tbody id="lw-users-tbody"></tbody>
                </table>
                <div class="lw-pager">
                    <button id="lw-users-prev" class="lw-btn lw-btn-text" type="button" disabled>
                        "Previous"
                    </button>
                    <span id="lw-users-page" class="lw-pager-label"></span>
                    <button id="lw-users-next" class="lw-btn lw-btn-text" type="button" disabled>
                        "Next"
                    </button>
                </div>
                <p id="lw-users-status" class="lw-settings-status"></p>
            </div>

            <div class="lw-modal-overlay" id="lw-add-user-modal" hidden=true>
                <div
                    class="lw-modal"
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby="lw-add-user-title"
                >
                    <h2 id="lw-add-user-title" class="lw-modal-title">"Add user"</h2>
                    <form id="lw-add-user-form">
                        <label class="lw-field">
                            "Username"
                            <input name="username" class="lw-input" autocomplete="off" required/>
                        </label>
                        <label class="lw-field">
                            "Email"
                            <input
                                name="email"
                                type="email"
                                class="lw-input"
                                autocomplete="off"
                                required
                            />
                        </label>
                        <label class="lw-field">
                            "Password"
                            <span class="lw-input-wrap">
                                <input
                                    name="password"
                                    type="password"
                                    class="lw-input"
                                    autocomplete="new-password"
                                    required
                                />
                                <button
                                    type="button"
                                    class="lw-eye-btn"
                                    aria-label="Show password"
                                >
                                    "👁"
                                </button>
                            </span>
                        </label>
                        <label class="lw-field">
                            "Confirm password"
                            <span class="lw-input-wrap">
                                <input
                                    name="password_confirm"
                                    type="password"
                                    class="lw-input"
                                    autocomplete="new-password"
                                    required
                                />
                                <button
                                    type="button"
                                    class="lw-eye-btn"
                                    aria-label="Show password"
                                >
                                    "👁"
                                </button>
                            </span>
                        </label>
                        <p class="lw-form-error" id="lw-add-user-error" hidden=true></p>
                        <div class="lw-modal-actions">
                            <button type="button" class="lw-btn lw-btn-text" data-close="">
                                "Cancel"
                            </button>
                            <button type="submit" class="lw-btn lw-btn-filled">"Create"</button>
                        </div>
                    </form>
                </div>
            </div>

            <script inner_html=SETTINGS_USERS_SCRIPT></script>
        </section>
    }
}

// Rows are built with createElement / textContent only — usernames and
// emails are user-derived and must never reach innerHTML.
const SETTINGS_USERS_SCRIPT: &str = r#"
(function () {
    const LIMIT = 20;
    let page = 1;
    let me = null;

    const el = id => document.getElementById(id);
    const root = el('lw-settings-users-root');
    const forbidden = el('lw-settings-forbidden');
    const tbody = el('lw-users-tbody');
    const prevBtn = el('lw-users-prev');
    const nextBtn = el('lw-users-next');
    const pageLabel = el('lw-users-page');
    const status = el('lw-users-status');
    const addModal = el('lw-add-user-modal');

    function onAuth(user) {
        me = user;
        if (!me || !me.admin) {
            forbidden.hidden = false;
            return;
        }
        root.hidden = false;
        loadPage(1);
    }
    if (window.lw && window.lw.ready) onAuth(window.lw.me);
    else document.addEventListener('lw-auth-ready', e => onAuth(e.detail));

    function loadPage(n) {
        page = n;
        status.textContent = '';
        fetch('/api/admin/users?page=' + n + '&limit=' + LIMIT, { headers: lw.authHeaders() })
            .then(r => {
                if (!r.ok) throw new Error('HTTP ' + r.status);
                return r.json();
            })
            .then(renderPage)
            .catch(err => { status.textContent = 'Failed to load users: ' + String(err); });
    }

    function renderPage(data) {
        const totalPages = Math.max(1, Math.ceil(data.total / data.limit));
        // Deleting the last row of a trailing page leaves it empty —
        // step back instead of rendering a blank table.
        if (data.users.length === 0 && page > 1) {
            loadPage(page - 1);
            return;
        }
        tbody.replaceChildren();
        data.users.forEach(u => tbody.appendChild(buildRow(u)));
        pageLabel.textContent = 'Page ' + data.page + ' of ' + totalPages;
        prevBtn.disabled = data.page <= 1;
        nextBtn.disabled = data.page >= totalPages;
    }

    function buildRow(u) {
        const isSelf = me && u.uuid === me.uuid;
        const tr = document.createElement('tr');

        const tdName = document.createElement('td');
        tdName.textContent = u.username + (isSelf ? ' (you)' : '');
        tr.appendChild(tdName);

        const tdEmail = document.createElement('td');
        tdEmail.textContent = u.email;
        tr.appendChild(tdEmail);

        const tdAdmin = document.createElement('td');
        const toggle = document.createElement('input');
        toggle.type = 'checkbox';
        toggle.checked = u.admin;
        // Mirrors the server guard: you can't demote yourself.
        toggle.disabled = isSelf;
        toggle.addEventListener('change', () => {
            fetch('/api/admin/users/' + encodeURIComponent(u.uuid), {
                method: 'PATCH',
                headers: Object.assign({ 'Content-Type': 'application/json' }, lw.authHeaders()),
                body: JSON.stringify({ admin: toggle.checked }),
            }).then(r => {
                if (!r.ok) throw new Error('HTTP ' + r.status);
            }).catch(err => {
                toggle.checked = !toggle.checked;
                status.textContent = 'Failed to update admin flag: ' + String(err);
            });
        });
        tdAdmin.appendChild(toggle);
        tr.appendChild(tdAdmin);

        const tdActions = document.createElement('td');
        const del = document.createElement('button');
        del.type = 'button';
        del.className = 'lw-btn lw-btn-danger';
        del.textContent = 'Delete';
        del.disabled = isSelf;
        del.addEventListener('click', () => {
            const ok = confirm(
                'Delete user "' + u.username + '"? ' +
                'Their settings and lore notes will be kept without an owner.');
            if (!ok) return;
            fetch('/api/admin/users/' + encodeURIComponent(u.uuid), {
                method: 'DELETE',
                headers: lw.authHeaders(),
            }).then(r => {
                if (!r.ok) throw new Error('HTTP ' + r.status);
                loadPage(page);
            }).catch(err => {
                status.textContent = 'Failed to delete user: ' + String(err);
            });
        });
        tdActions.appendChild(del);
        tr.appendChild(tdActions);

        return tr;
    }

    prevBtn.addEventListener('click', () => loadPage(page - 1));
    nextBtn.addEventListener('click', () => loadPage(page + 1));

    // ── add-user modal ───────────────────────────────────────────────
    el('lw-users-add').addEventListener('click', () => { addModal.hidden = false; });
    addModal.addEventListener('click', e => {
        if (e.target === addModal) addModal.hidden = true;
    });
    addModal.querySelectorAll('[data-close]').forEach(b =>
        b.addEventListener('click', () => { addModal.hidden = true; }));
    document.addEventListener('keydown', e => {
        if (e.key === 'Escape') addModal.hidden = true;
    });

    el('lw-add-user-form').addEventListener('submit', e => {
        e.preventDefault();
        const f = new FormData(e.target);
        const errEl = el('lw-add-user-error');
        errEl.hidden = true;
        if (f.get('password') !== f.get('password_confirm')) {
            errEl.textContent = 'Passwords do not match';
            errEl.hidden = false;
            return;
        }
        fetch('/api/admin/users', {
            method: 'POST',
            headers: Object.assign({ 'Content-Type': 'application/json' }, lw.authHeaders()),
            body: JSON.stringify({
                username: f.get('username'),
                email: f.get('email'),
                password: f.get('password'),
            }),
        }).then(r => {
            if (r.ok) {
                addModal.hidden = true;
                e.target.reset();
                loadPage(page);
                return;
            }
            return r.json()
                .catch(() => { throw new Error('HTTP ' + r.status); })
                .then(body => { throw body; });
        }).catch(err => {
            errEl.textContent = (err && err.message) ? err.message : 'Request failed';
            errEl.hidden = false;
        });
    });
})();
"#;
