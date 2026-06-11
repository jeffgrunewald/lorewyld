use leptos::prelude::*;

/// Right-aligned auth area in the server header. Every element starts
/// hidden; [`AUTH_SCRIPT`] reveals the correct set after probing
/// `/api/users/me`, so there's no logged-in/logged-out flash.
#[component]
pub fn HeaderAuth() -> impl IntoView {
    view! {
        <div class="lw-header-auth">
            <button id="lw-btn-register" class="lw-btn lw-btn-outline" type="button" hidden=true>
                "Register"
            </button>
            <button id="lw-btn-login" class="lw-btn lw-btn-filled" type="button" hidden=true>
                "Log in"
            </button>
            <button
                id="lw-header-username"
                class="lw-header-username"
                type="button"
                title="Change password"
                hidden=true
            ></button>
            <button id="lw-btn-logout" class="lw-btn lw-btn-filled" type="button" hidden=true>
                "Log out"
            </button>
        </div>
    }
}

/// Login + registration modal dialogs, rendered (hidden) on every page.
/// All dynamic text is set via `textContent` in [`AUTH_SCRIPT`] —
/// consistent with the codebase's XSS-safe pure-DOM convention.
#[component]
pub fn AuthModals() -> impl IntoView {
    view! {
        <div class="lw-modal-overlay" id="lw-login-modal" hidden=true>
            <div class="lw-modal" role="dialog" aria-modal="true" aria-labelledby="lw-login-title">
                <h2 id="lw-login-title" class="lw-modal-title">"Log in"</h2>
                <form id="lw-login-form">
                    <label class="lw-field">
                        "Username"
                        <input name="username" class="lw-input" autocomplete="username" required/>
                    </label>
                    <label class="lw-field">
                        "Password"
                        <span class="lw-input-wrap">
                            <input
                                name="password"
                                type="password"
                                class="lw-input"
                                autocomplete="current-password"
                                required
                            />
                            <button type="button" class="lw-eye-btn" aria-label="Show password">
                                "👁"
                            </button>
                        </span>
                    </label>
                    <p class="lw-form-error" id="lw-login-error" hidden=true></p>
                    <div class="lw-modal-actions">
                        <button type="button" class="lw-btn lw-btn-text" data-close="">
                            "Cancel"
                        </button>
                        <button type="submit" class="lw-btn lw-btn-filled">"Log in"</button>
                    </div>
                </form>
            </div>
        </div>

        <div class="lw-modal-overlay" id="lw-register-modal" hidden=true>
            <div
                class="lw-modal"
                role="dialog"
                aria-modal="true"
                aria-labelledby="lw-register-title"
            >
                <h2 id="lw-register-title" class="lw-modal-title">"Register"</h2>
                <form id="lw-register-form">
                    <label class="lw-field">
                        "Username"
                        <input name="username" class="lw-input" autocomplete="username" required/>
                    </label>
                    <label class="lw-field">
                        "Email"
                        <input
                            name="email"
                            type="email"
                            class="lw-input"
                            autocomplete="email"
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
                            <button type="button" class="lw-eye-btn" aria-label="Show password">
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
                            <button type="button" class="lw-eye-btn" aria-label="Show password">
                                "👁"
                            </button>
                        </span>
                    </label>
                    <label class="lw-field">
                        "Server join code"
                        <input name="join_code" class="lw-input" autocomplete="off" required/>
                    </label>
                    <p class="lw-form-error" id="lw-register-error" hidden=true></p>
                    <div class="lw-modal-actions">
                        <button type="button" class="lw-btn lw-btn-text" data-close="">
                            "Cancel"
                        </button>
                        <button type="submit" class="lw-btn lw-btn-filled">"Register"</button>
                    </div>
                </form>
            </div>
        </div>

        <div class="lw-modal-overlay" id="lw-password-modal" hidden=true>
            <div
                class="lw-modal"
                role="dialog"
                aria-modal="true"
                aria-labelledby="lw-password-title"
            >
                <h2 id="lw-password-title" class="lw-modal-title">"Change password"</h2>
                <form id="lw-password-form">
                    <label class="lw-field">
                        "Current password"
                        <span class="lw-input-wrap">
                            <input
                                name="current_password"
                                type="password"
                                class="lw-input"
                                autocomplete="current-password"
                                required
                            />
                            <button type="button" class="lw-eye-btn" aria-label="Show password">
                                "👁"
                            </button>
                        </span>
                    </label>
                    <label class="lw-field">
                        "New password"
                        <span class="lw-input-wrap">
                            <input
                                name="new_password"
                                type="password"
                                class="lw-input"
                                autocomplete="new-password"
                                required
                            />
                            <button type="button" class="lw-eye-btn" aria-label="Show password">
                                "👁"
                            </button>
                        </span>
                    </label>
                    <label class="lw-field">
                        "Confirm new password"
                        <span class="lw-input-wrap">
                            <input
                                name="new_password_confirm"
                                type="password"
                                class="lw-input"
                                autocomplete="new-password"
                                required
                            />
                            <button type="button" class="lw-eye-btn" aria-label="Show password">
                                "👁"
                            </button>
                        </span>
                    </label>
                    <p class="lw-form-error" id="lw-password-error" hidden=true></p>
                    <p class="lw-form-success" id="lw-password-success" hidden=true>
                        "Password updated."
                    </p>
                    <div class="lw-modal-actions">
                        <button type="button" class="lw-btn lw-btn-text" data-close="">
                            "Close"
                        </button>
                        <button type="submit" class="lw-btn lw-btn-filled">
                            "Update password"
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}

/// Session bootstrap + auth UI wiring, run on every page load. Exposes
/// `window.lw` helpers (`token()`, `authHeaders()`, `me`) for other page
/// scripts and dispatches a `lw-auth-ready` CustomEvent (detail = the
/// `/api/users/me` payload or null) once the session probe settles.
pub const AUTH_SCRIPT: &str = r#"
(function () {
    const TOKEN_KEY = 'lw_session_token';

    const lw = {
        token: () => localStorage.getItem(TOKEN_KEY),
        authHeaders: () => {
            const t = localStorage.getItem(TOKEN_KEY);
            return t ? { 'Authorization': 'Bearer ' + t } : {};
        },
        me: null,
        ready: false,
    };
    window.lw = lw;

    const el = id => document.getElementById(id);
    const btnRegister = el('lw-btn-register');
    const btnLogin = el('lw-btn-login');
    const btnLogout = el('lw-btn-logout');
    const usernameEl = el('lw-header-username');
    const navSettings = el('lw-nav-settings');

    function announce(me) {
        lw.me = me;
        lw.ready = true;
        document.dispatchEvent(new CustomEvent('lw-auth-ready', { detail: me }));
    }

    function showLoggedOut() {
        btnRegister.hidden = false;
        btnLogin.hidden = false;
        usernameEl.hidden = true;
        btnLogout.hidden = true;
    }

    function showLoggedIn(me) {
        btnRegister.hidden = true;
        btnLogin.hidden = true;
        usernameEl.textContent = me.username;
        usernameEl.hidden = false;
        btnLogout.hidden = false;
        if (navSettings && me.admin) navSettings.hidden = false;
    }

    const token = lw.token();
    if (token) {
        fetch('/api/users/me', { headers: lw.authHeaders() })
            .then(r => {
                if (!r.ok) throw new Error('HTTP ' + r.status);
                return r.json();
            })
            .then(me => {
                showLoggedIn(me);
                announce(me);
            })
            .catch(() => {
                localStorage.removeItem(TOKEN_KEY);
                showLoggedOut();
                announce(null);
            });
    } else {
        showLoggedOut();
        announce(null);
    }

    // ── modal plumbing ───────────────────────────────────────────────
    function wireModal(overlay) {
        overlay.addEventListener('click', e => {
            if (e.target === overlay) overlay.hidden = true;
        });
        overlay.querySelectorAll('[data-close]').forEach(b =>
            b.addEventListener('click', () => { overlay.hidden = true; }));
    }

    const loginModal = el('lw-login-modal');
    const registerModal = el('lw-register-modal');
    const passwordModal = el('lw-password-modal');
    wireModal(loginModal);
    wireModal(registerModal);
    wireModal(passwordModal);

    document.addEventListener('keydown', e => {
        if (e.key === 'Escape') {
            loginModal.hidden = true;
            registerModal.hidden = true;
            passwordModal.hidden = true;
        }
    });

    btnLogin.addEventListener('click', () => { loginModal.hidden = false; });
    btnRegister.addEventListener('click', () => { registerModal.hidden = false; });
    usernameEl.addEventListener('click', () => {
        el('lw-password-error').hidden = true;
        el('lw-password-success').hidden = true;
        passwordModal.hidden = false;
    });

    btnLogout.addEventListener('click', () => {
        fetch('/api/users/logout', { method: 'POST', headers: lw.authHeaders() })
            .finally(() => {
                localStorage.removeItem(TOKEN_KEY);
                location.reload();
            });
    });

    // Password visibility toggles (shared with any page that uses
    // .lw-eye-btn next to an input).
    document.querySelectorAll('.lw-eye-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const input = btn.parentElement.querySelector('input');
            if (!input) return;
            const masked = input.type === 'password';
            input.type = masked ? 'text' : 'password';
            btn.setAttribute('aria-label', masked ? 'Hide password' : 'Show password');
        });
    });

    function showFormError(errEl, err) {
        errEl.textContent = (err && err.message) ? err.message : 'Request failed';
        errEl.hidden = false;
    }

    function postJson(url, body) {
        return fetch(url, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body),
        }).then(r => {
            if (r.ok) return r.json();
            return r.json()
                .catch(() => { throw new Error('HTTP ' + r.status); })
                .then(errBody => { throw errBody; });
        });
    }

    function storeSessionAndReload(data) {
        localStorage.setItem(TOKEN_KEY, data.session_token);
        location.reload();
    }

    el('lw-login-form').addEventListener('submit', e => {
        e.preventDefault();
        const f = new FormData(e.target);
        const errEl = el('lw-login-error');
        errEl.hidden = true;
        postJson('/api/users/login', {
            username: f.get('username'),
            password: f.get('password'),
        })
            .then(storeSessionAndReload)
            .catch(err => showFormError(errEl, err));
    });

    el('lw-register-form').addEventListener('submit', e => {
        e.preventDefault();
        const f = new FormData(e.target);
        const errEl = el('lw-register-error');
        errEl.hidden = true;
        if (f.get('password') !== f.get('password_confirm')) {
            showFormError(errEl, { message: 'Passwords do not match' });
            return;
        }
        postJson('/api/users/register', {
            join_code: f.get('join_code'),
            username: f.get('username'),
            email: f.get('email'),
            password: f.get('password'),
        })
            .then(storeSessionAndReload)
            .catch(err => showFormError(errEl, err));
    });

    el('lw-password-form').addEventListener('submit', e => {
        e.preventDefault();
        const f = new FormData(e.target);
        const errEl = el('lw-password-error');
        const okEl = el('lw-password-success');
        errEl.hidden = true;
        okEl.hidden = true;
        if (f.get('new_password') !== f.get('new_password_confirm')) {
            showFormError(errEl, { message: 'New passwords do not match' });
            return;
        }
        if (String(f.get('new_password')).length < 8) {
            showFormError(errEl, { message: 'Password must be at least 8 characters' });
            return;
        }
        fetch('/api/users/password', {
            method: 'POST',
            headers: Object.assign({ 'Content-Type': 'application/json' }, lw.authHeaders()),
            body: JSON.stringify({
                current_password: f.get('current_password'),
                new_password: f.get('new_password'),
            }),
        }).then(r => {
            if (r.ok) {
                e.target.reset();
                okEl.hidden = false;
                return;
            }
            return r.json()
                .catch(() => { throw new Error('HTTP ' + r.status); })
                .then(body => { throw body; });
        }).catch(err => showFormError(errEl, err));
    });
})();
"#;
