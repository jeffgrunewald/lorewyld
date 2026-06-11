use leptos::prelude::*;

/// Admin server-settings page: editable server name, read-only join
/// code with a regenerate action, and the read-only software version.
/// Gated like the users page — the admin API endpoints are the real
/// access control.
#[component]
pub fn SettingsServerPage() -> impl IntoView {
    view! {
        <section class="lw-settings">
            <p id="lw-settings-forbidden" class="lw-settings-forbidden" hidden=true>
                "You must be logged in as an administrator to view this page."
            </p>
            <div id="lw-settings-server-root" hidden=true>
                <header class="lw-settings-header">
                    <h1 class="lw-settings-title">"Server"</h1>
                </header>
                <form id="lw-server-form" class="lw-settings-form">
                    <label class="lw-field">
                        "Server name"
                        <input name="name" id="lw-server-name" class="lw-input" required/>
                    </label>
                    <div class="lw-field">
                        "Join code"
                        <div class="lw-joincode-row">
                            <code id="lw-server-join-code" class="lw-settings-joincode"></code>
                            <button
                                type="button"
                                id="lw-server-regen-code"
                                class="lw-btn lw-btn-text"
                            >
                                "Regenerate"
                            </button>
                        </div>
                    </div>
                    <div class="lw-field">
                        "Software version"
                        <p id="lw-server-version" class="lw-settings-version"></p>
                    </div>
                    <div class="lw-modal-actions">
                        <button type="submit" class="lw-btn lw-btn-filled">"Save"</button>
                    </div>
                </form>
                <p id="lw-server-status" class="lw-settings-status"></p>
            </div>
            <script inner_html=SETTINGS_SERVER_SCRIPT></script>
        </section>
    }
}

const SETTINGS_SERVER_SCRIPT: &str = r#"
(function () {
    const el = id => document.getElementById(id);
    const root = el('lw-settings-server-root');
    const forbidden = el('lw-settings-forbidden');
    const nameInput = el('lw-server-name');
    const joinCodeEl = el('lw-server-join-code');
    const versionEl = el('lw-server-version');
    const status = el('lw-server-status');

    function onAuth(me) {
        if (!me || !me.admin) {
            forbidden.hidden = false;
            return;
        }
        root.hidden = false;
        load();
    }
    if (window.lw && window.lw.ready) onAuth(window.lw.me);
    else document.addEventListener('lw-auth-ready', e => onAuth(e.detail));

    function render(s) {
        nameInput.value = s.name;
        joinCodeEl.textContent = s.join_code;
        versionEl.textContent = s.version;
    }

    function load() {
        fetch('/api/admin/server', { headers: lw.authHeaders() })
            .then(r => {
                if (!r.ok) throw new Error('HTTP ' + r.status);
                return r.json();
            })
            .then(render)
            .catch(err => { status.textContent = 'Failed to load settings: ' + String(err); });
    }

    el('lw-server-form').addEventListener('submit', e => {
        e.preventDefault();
        status.textContent = '';
        fetch('/api/admin/server', {
            method: 'PATCH',
            headers: Object.assign({ 'Content-Type': 'application/json' }, lw.authHeaders()),
            body: JSON.stringify({ name: nameInput.value }),
        }).then(r => {
            if (r.ok) return r.json();
            return r.json()
                .catch(() => { throw new Error('HTTP ' + r.status); })
                .then(body => { throw body; });
        }).then(s => {
            render(s);
            status.textContent = 'Saved.';
        }).catch(err => {
            status.textContent = 'Failed to save: ' +
                ((err && err.message) ? err.message : String(err));
        });
    });

    el('lw-server-regen-code').addEventListener('click', () => {
        const ok = confirm(
            'Regenerate the join code? The current code will stop working immediately.');
        if (!ok) return;
        status.textContent = '';
        fetch('/api/admin/server/join-code', {
            method: 'POST',
            headers: lw.authHeaders(),
        }).then(r => {
            if (r.ok) return r.json();
            return r.json()
                .catch(() => { throw new Error('HTTP ' + r.status); })
                .then(body => { throw body; });
        }).then(s => {
            render(s);
            status.textContent = 'Join code regenerated.';
        }).catch(err => {
            status.textContent = 'Failed to regenerate: ' +
                ((err && err.message) ? err.message : String(err));
        });
    });
})();
"#;
