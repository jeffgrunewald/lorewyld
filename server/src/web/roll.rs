use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiceType {
    D4,
    D6,
    D8,
    D10,
    D100,
    D12,
    D20,
}

impl DiceType {
    pub const fn all() -> &'static [Self] {
        &[
            Self::D4,
            Self::D6,
            Self::D8,
            Self::D10,
            Self::D100,
            Self::D12,
            Self::D20,
        ]
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::D4 => "d4",
            Self::D6 => "d6",
            Self::D8 => "d8",
            Self::D10 => "d10",
            Self::D100 => "d100",
            Self::D12 => "d12",
            Self::D20 => "d20",
        }
    }

    pub const fn sides(self) -> u32 {
        match self {
            Self::D4 => 4,
            Self::D6 => 6,
            Self::D8 => 8,
            Self::D10 => 10,
            Self::D100 => 100,
            Self::D12 => 12,
            Self::D20 => 20,
        }
    }
}

#[component]
pub fn Roll() -> impl IntoView {
    let rows = DiceType::all()
        .iter()
        .map(|dt| {
            let label = dt.label();
            let sides = dt.sides();
            let img = format!("/assets/dice/{label}.png");
            view! {
                <div class="lw-roll-row">
                    <button
                        class="lw-roll-die-btn"
                        type="button"
                        data-die=label
                        data-sides=sides
                        aria-label=label
                    >
                        <img src=img alt=label/>
                    </button>
                    <span class="lw-roll-output" data-die-output=label></span>
                </div>
            }
        })
        .collect_view();

    view! {
        <section class="lw-roll">
            <div class="lw-roll-dice-col">
                {rows}
            </div>
            <div id="lw-grand-total" class="lw-roll-grand-total"></div>
            <button id="lw-roll-action" class="lw-roll-btn" type="button" disabled>
                "Roll"
            </button>
            <script inner_html=ROLL_SCRIPT></script>
        </section>
    }
}

const ROLL_SCRIPT: &str = r#"
(function () {
    const dice = Array.from(document.querySelectorAll('[data-die]'));
    const outputs = new Map();
    document.querySelectorAll('[data-die-output]').forEach(el => {
        outputs.set(el.dataset.dieOutput, el);
    });
    const action = document.getElementById('lw-roll-action');
    const total = document.getElementById('lw-grand-total');

    const queue = Object.create(null);
    const rolls = Object.create(null);
    let rolled = false;

    function rollOne(sides) {
        const buf = new Uint32Array(1);
        const cap = Math.floor(0x100000000 / sides) * sides;
        while (true) {
            crypto.getRandomValues(buf);
            if (buf[0] < cap) return (buf[0] % sides) + 1;
        }
    }

    function render() {
        let grand = 0;
        let anyQueued = false;
        for (const btn of dice) {
            const key = btn.dataset.die;
            const count = queue[key] || 0;
            if (count > 0) anyQueued = true;
            const out = outputs.get(key);
            if (rolled && rolls[key]) {
                const sub = rolls[key].reduce((a, b) => a + b, 0);
                grand += sub;
                out.textContent = rolls[key].join(', ') + ' = ' + sub;
            } else if (count > 0) {
                out.textContent = 'x' + count;
            } else {
                out.textContent = '';
            }
            btn.disabled = rolled;
        }
        if (rolled) {
            total.textContent = 'Total: ' + grand;
            total.classList.add('lw-visible');
            action.textContent = 'Clear';
            action.disabled = false;
        } else {
            total.textContent = '';
            total.classList.remove('lw-visible');
            action.textContent = 'Roll';
            action.disabled = !anyQueued;
        }
    }

    for (const btn of dice) {
        btn.addEventListener('click', () => {
            if (rolled) return;
            const key = btn.dataset.die;
            const next = Math.min((queue[key] || 0) + 1, 99);
            queue[key] = next;
            render();
        });
    }

    action.addEventListener('click', () => {
        if (rolled) {
            for (const k of Object.keys(queue)) delete queue[k];
            for (const k of Object.keys(rolls)) delete rolls[k];
            rolled = false;
        } else {
            let any = false;
            for (const btn of dice) {
                const key = btn.dataset.die;
                const count = queue[key] || 0;
                if (count <= 0) continue;
                any = true;
                const sides = parseInt(btn.dataset.sides, 10);
                const out = [];
                for (let i = 0; i < count; i++) out.push(rollOne(sides));
                rolls[key] = out;
            }
            if (!any) return;
            rolled = true;
        }
        render();
    });

    render();
})();
"#;
