use crate::{clx, cn};
use leptos::prelude::*;

clx! {
    /// Page footer landmark. Wrap the footer content and let the caller supply
    /// layout via composed sections.
    Footer, footer, ""
}
clx! {
    /// Centered max-width column that constrains footer content.
    FooterContainer, div, "px-6 mx-auto max-w-5xl"
}
clx! {
    /// Top-level footer region; gains spacing when bordered top or bottom.
    FooterSection, section,
    "w-full max-w-5xl mx-auto py-6 flex flex-wrap gap-4 justify-between items-center",
    "[.border-b]:mb-14",
    "[.border-t]:mt-14"
}
clx! {
    /// Primary footer grid arranging brand and link sections.
    FooterGrid, div, "grid gap-12 md:grid-cols-5"
}
clx! {
    /// Brand block spanning two columns on wide layouts.
    FooterBrand, div, "md:col-span-2"
}
clx! {
    /// Inline-block anchor sized to its brand mark.
    FooterBrandLink, a, "block size-fit"
}
clx! {
    /// Muted brand description text.
    FooterDescription, p, "text-sm text-foreground/70 text-balance"
}
clx! {
    /// Grid wrapping the columns of footer link sections.
    FooterSectionsGrid, div, "grid gap-6"
}
clx! {
    /// A single column of related footer links.
    FooterLinksSection, div, "space-y-4 text-sm"
}
clx! {
    /// Heading for a footer link column.
    FooterTitle, span, "block font-medium"
}
clx! {
    /// Vertical-stacking list of footer links.
    FooterLinks, div, "flex flex-wrap gap-4 sm:flex-col"
}
clx! {
    /// Muted footer link with a hover accent.
    FooterLink, a, "block duration-150 text-foreground/70 hover:text-primary"
}
clx! {
    /// Row of social or icon links.
    FooterSocialContainer, div, "flex flex-wrap gap-6 text-sm"
}
clx! {
    /// Centered horizontal navigation row.
    FooterNavContainer, div, "flex flex-wrap gap-6 justify-center my-8 text-sm"
}
clx! {
    /// Muted copyright line.
    FooterCopyright, small, "text-sm text-foreground/70"
}

/// External footer link that opens in a new tab. The `target`/`rel` pair is
/// fixed for safe cross-origin navigation; `href`, `aria-label` and other
/// native attributes are forwarded to the underlying `<a>`.
#[component]
pub fn FooterExternalLink(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <a
            data-name="FooterExternalLink"
            target="_blank"
            rel="noopener noreferrer"
            class=move || cn!("block text-foreground/70 hover:text-primary", class.get())
        >
            {children()}
        </a>
    }
}
