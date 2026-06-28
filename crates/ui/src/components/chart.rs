//! chart — the recharts-independent parts of shadcn's `chart`. The upstream wraps
//! recharts (a React charting library) for the actual axes/series rendering, which has
//! no pure-Leptos equivalent and is out of scope (no JS dependency is added). What IS
//! transcribable — and what makes the kit "chart-ready" — is ported here: the
//! [`ChartContainer`] (themed, responsive box) and [`ChartStyle`] (the `--color-<key>`
//! CSS variables from a [`ChartConfig`]). Render any chart (e.g. a hand-authored SVG)
//! inside the container; its series read the injected `--color-<key>` variables.

use crate::cn;
use leptos::prelude::*;

/// One series' theming: its data `key`, display `label`, and `color` (any CSS color).
#[derive(Clone, Debug)]
pub struct ChartSeries {
    /// The series key — drives `--color-<key>`.
    pub key: String,
    /// The human-readable label (for a legend/tooltip).
    pub label: String,
    /// The series color (any CSS color, e.g. `var(--chart-1)`).
    pub color: String,
}

/// Chart theming config — one [`ChartSeries`] per data key.
pub type ChartConfig = Vec<ChartSeries>;

const CHART_CONTAINER_CLASS: &str = "cn-chart flex aspect-video justify-center text-xs [&_.recharts-cartesian-axis-tick_text]:fill-muted-foreground [&_.recharts-cartesian-grid_line[stroke='#ccc']]:stroke-border/50 [&_.recharts-curve.recharts-tooltip-cursor]:stroke-border [&_.recharts-dot[stroke='#fff']]:stroke-transparent [&_.recharts-layer]:outline-hidden [&_.recharts-polar-grid_[stroke='#ccc']]:stroke-border [&_.recharts-radial-bar-background-sector]:fill-muted [&_.recharts-rectangle.recharts-tooltip-cursor]:fill-muted [&_.recharts-reference-line_[stroke='#ccc']]:stroke-border [&_.recharts-sector]:outline-hidden [&_.recharts-sector[stroke='#fff']]:stroke-transparent [&_.recharts-surface]:outline-hidden";

/// Injects the `--color-<key>` custom properties for a [`ChartConfig`], scoped to
/// `[data-chart=<id>]` for both light and `.dark`. Transcribed from shadcn's
/// `ChartStyle` (which used `dangerouslySetInnerHTML`).
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned; read once to build the scoped CSS"
)]
pub fn ChartStyle(#[prop(into)] id: String, config: ChartConfig) -> impl IntoView {
    let vars: String = config
        .iter()
        .filter(|series| !series.color.is_empty())
        .map(|series| format!("  --color-{}: {};", series.key, series.color))
        .collect::<Vec<_>>()
        .join("\n");
    let css = if vars.is_empty() {
        String::new()
    } else {
        format!("[data-chart={id}] {{\n{vars}\n}}\n.dark [data-chart={id}] {{\n{vars}\n}}\n")
    };
    view! { <style inner_html=css></style> }
}

/// ChartContainer — shadcn `chart` container: a themed, aspect-ratio box that injects
/// the series color variables ([`ChartStyle`]) and hosts the chart. `id` scopes the
/// CSS variables; render the chart itself (any SVG/markup) as `children`.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned; `id` is read once to build the chart id"
)]
pub fn ChartContainer(
    #[prop(into)] id: String,
    config: ChartConfig,
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let chart_id = format!("chart-{id}");
    let style_id = chart_id.clone();
    view! {
        <div
            data-slot="chart"
            data-chart=chart_id
            class=move || cn!(CHART_CONTAINER_CLASS, class.get())
        >
            <ChartStyle id=style_id config=config />
            {children()}
        </div>
    }
}
