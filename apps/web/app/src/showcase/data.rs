use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use leptos::prelude::*;
use leptos_icons::Icon;
use strum::{Display, EnumIter, IntoEnumIterator};
use ui::{
    Badge, BadgeVariant, Card, CardAction, CardCarousel, CardCarouselIndicator,
    CardCarouselIndicators, CardCarouselNav, CardCarouselNavButton, CardCarouselOverlay,
    CardCarouselSlide, CardCarouselTrack, CardContent, CardDescription, CardFooter, CardHeader,
    CardItem, CardList, CardSize, CardTitle, Carousel, CarouselContent, CarouselIndicator,
    CarouselItem, CarouselNext, CarouselOrientation, CarouselPrevious, Checkbox, DataGridColumn,
    DataGridRow, DataGridToolbar, DataTableContainer, DataTableFooter, EditableCellContent,
    GenericGridHeader, GridCell, GridPinnedCell, GridRow, GridSelectCell, Item, ItemActions,
    ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemMedia, ItemMediaVariant,
    ItemSeparator, ItemSize, ItemTitle, ItemVariant, PinnableColumn, SortDirection, SortableColumn,
    Table, TableBody, TableCaption, TableCell, TableHead, TableHeader, TableRow, VirtualFor,
    VirtualizedGrid, VirtualizedGridBody, generate_grid_style, use_cell_edit,
};

use super::{Demo, Page, Section};

/// Tables, the virtualized data grid, carousels and cards.
#[component]
pub fn DataPage() -> impl IntoView {
    view! {
        <Page title="Data" subtitle="Tables, the virtualized data grid, carousels and cards.">
            <TableSection />
            <CardSection />
            <ItemSection />
            <CarouselSection />
            <CardCarouselSection />
            <DataGridSection />
        </Page>
    }
}

// ---------------------------------------------------------------------------
// Table
// ---------------------------------------------------------------------------

#[component]
fn TableSection() -> impl IntoView {
    let rows = [
        ("Aurora HQ", "London", 48, "Open"),
        ("Beacon Studio", "Berlin", 24, "Open"),
        ("Cobalt Lab", "Lisbon", 16, "Maintenance"),
        ("Delta Annex", "Madrid", 32, "Open"),
    ];
    let total: i32 = rows.iter().map(|(_, _, desks, _)| *desks).sum();

    let body = rows
        .into_iter()
        .map(|(site, city, desks, status)| {
            view! {
                <TableRow>
                    <TableCell class="font-medium">{site}</TableCell>
                    <TableCell>{city}</TableCell>
                    <TableCell class="text-right">{desks.to_string()}</TableCell>
                    <TableCell class="text-right">{status}</TableCell>
                </TableRow>
            }
        })
        .collect_view();

    view! {
        <Section
            title="Table"
            description="Semantic table primitives inside the horizontal-scroll container, with a caption and a totals footer."
        >
            <Demo>
                <DataTableContainer>
                    <Table>
                        <TableCaption>"Desk inventory by site"</TableCaption>
                        <TableHeader>
                            <TableRow>
                                <TableHead>"Site"</TableHead>
                                <TableHead>"City"</TableHead>
                                <TableHead class="text-right">"Desks"</TableHead>
                                <TableHead class="text-right">"Status"</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>{body}</TableBody>
                        <DataTableFooter>
                            <TableRow>
                                <TableCell>"Total"</TableCell>
                                <TableCell></TableCell>
                                <TableCell class="text-right">{total.to_string()}</TableCell>
                                <TableCell></TableCell>
                            </TableRow>
                        </DataTableFooter>
                    </Table>
                </DataTableContainer>
            </Demo>
        </Section>
    }
}

// ---------------------------------------------------------------------------
// Card
// ---------------------------------------------------------------------------

#[component]
fn CardSection() -> impl IntoView {
    view! {
        <Section
            title="Card"
            description="Surface container with header, action slot, content, footer and an inline item list."
        >
            <Demo>
                <Card class="w-80">
                    <CardHeader>
                        <CardTitle>"Booking summary"</CardTitle>
                        <CardDescription>"Desk 14B · Aurora HQ, London"</CardDescription>
                        <CardAction>
                            <Badge variant=BadgeVariant::Success>"Confirmed"</Badge>
                        </CardAction>
                    </CardHeader>
                    <CardContent>
                        <CardList>
                            <CardItem>
                                <Icon icon=icondata::LuCalendar attr:class="mr-2" />
                                "Mon 30 Jun, 09:00 – 17:00"
                            </CardItem>
                            <CardItem>
                                <Icon icon=icondata::LuLayoutGrid attr:class="mr-2" />
                                "Floor 3 · West wing"
                            </CardItem>
                        </CardList>
                    </CardContent>
                    <CardFooter class="border-t">
                        <CardDescription>"Cancel up to 1 hour before"</CardDescription>
                    </CardFooter>
                </Card>

                <Card size=CardSize::Sm class="w-72">
                    <CardHeader>
                        <CardTitle>"Compact card"</CardTitle>
                        <CardDescription>"Sm size — tighter rhythm and padding."</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <p class="text-sm text-muted-foreground">
                            "Use the small size in dense dashboards and side panels."
                        </p>
                    </CardContent>
                </Card>
            </Demo>
        </Section>
    }
}

// ---------------------------------------------------------------------------
// Item
// ---------------------------------------------------------------------------

#[component]
fn ItemSection() -> impl IntoView {
    view! {
        <Section
            title="Item"
            description="Interactive list rows: variants, sizes, media slots, header/footer bands and separators."
        >
            <Demo col=true label="Grouped rows with separators">
                <ItemGroup class="w-full max-w-xl rounded-lg border">
                    <Item size=ItemSize::Sm>
                        <ItemMedia variant=ItemMediaVariant::Icon>
                            <Icon icon=icondata::LuLayoutDashboard />
                        </ItemMedia>
                        <ItemContent>
                            <ItemTitle>"Overview"</ItemTitle>
                            <ItemDescription>"Live occupancy across every site."</ItemDescription>
                        </ItemContent>
                        <ItemActions>
                            <Icon icon=icondata::LuChevronRight attr:class="size-4" />
                        </ItemActions>
                    </Item>
                    <ItemSeparator />
                    <Item size=ItemSize::Sm>
                        <ItemMedia variant=ItemMediaVariant::Icon>
                            <Icon icon=icondata::LuTable />
                        </ItemMedia>
                        <ItemContent>
                            <ItemTitle>"Reports"</ItemTitle>
                            <ItemDescription>"Export booking history as CSV."</ItemDescription>
                        </ItemContent>
                        <ItemActions>
                            <Icon icon=icondata::LuChevronRight attr:class="size-4" />
                        </ItemActions>
                    </Item>
                </ItemGroup>
            </Demo>

            <Demo col=true label="Variants & sizes">
                <Item variant=ItemVariant::Outline href="/ui/data".to_string()>
                    <ItemMedia variant=ItemMediaVariant::Icon>
                        <Icon icon=icondata::LuCompass />
                    </ItemMedia>
                    <ItemContent>
                        <ItemTitle>"Outline · default size · link"</ItemTitle>
                        <ItemDescription>"Renders as an anchor when given href."</ItemDescription>
                    </ItemContent>
                </Item>
                <Item variant=ItemVariant::Muted size=ItemSize::Xs>
                    <ItemMedia>
                        <Icon icon=icondata::LuLayers attr:class="size-4" />
                    </ItemMedia>
                    <ItemContent>
                        <ItemTitle>"Muted · xs size"</ItemTitle>
                    </ItemContent>
                    <ItemActions>
                        <Badge variant=BadgeVariant::Muted size=ui::BadgeSize::Sm>
                            "new"
                        </Badge>
                    </ItemActions>
                </Item>
            </Demo>

            <Demo col=true label="Header / footer bands">
                <Item variant=ItemVariant::Outline class="w-full max-w-xl">
                    <ItemHeader>
                        <span class="text-sm font-medium">"Aurora HQ"</span>
                        <Badge variant=BadgeVariant::Outline>"London"</Badge>
                    </ItemHeader>
                    <ItemMedia variant=ItemMediaVariant::Image>
                        <img src="data:image/svg+xml;utf8,%3Csvg xmlns='http://www.w3.org/2000/svg' width='40' height='40'%3E%3Crect width='40' height='40' fill='%236366f1'/%3E%3C/svg%3E" />
                    </ItemMedia>
                    <ItemContent>
                        <ItemTitle>"Flagship workspace"</ItemTitle>
                        <ItemDescription>"48 desks across three floors."</ItemDescription>
                    </ItemContent>
                    <ItemFooter>
                        <span class="text-xs text-muted-foreground">"Updated 5m ago"</span>
                        <span class="text-xs text-muted-foreground">"32 / 48 booked"</span>
                    </ItemFooter>
                </Item>
            </Demo>
        </Section>
    }
}

// ---------------------------------------------------------------------------
// Carousel (reactive, no-JS)
// ---------------------------------------------------------------------------

#[component]
fn CarouselSection() -> impl IntoView {
    let slide = |label: &str, tint: &str| {
        let label = label.to_string();
        let class = format!(
            "flex h-40 w-full items-center justify-center rounded-md text-lg font-semibold {tint}"
        );
        view! { <div class=class>{label}</div> }
    };

    view! {
        <Section
            title="Carousel"
            description="Reactive slide carousel — arrow keys move between slides, with previous/next controls and a live indicator."
        >
            <Demo col=true label="Horizontal">
                <div class="px-12 w-full max-w-md">
                    <Carousel>
                        <CarouselContent>
                            <CarouselItem>
                                {slide("Slide 1", "bg-primary/15 text-primary")}
                            </CarouselItem>
                            <CarouselItem>
                                {slide("Slide 2", "bg-accent text-accent-foreground")}
                            </CarouselItem>
                            <CarouselItem>
                                {slide("Slide 3", "bg-muted text-muted-foreground")}
                            </CarouselItem>
                        </CarouselContent>
                        <CarouselPrevious />
                        <CarouselNext />
                        <CarouselIndicator />
                    </Carousel>
                </div>
            </Demo>

            <Demo col=true label="Vertical (looping)">
                <div class="px-12 py-12 w-full max-w-md">
                    <Carousel orientation=CarouselOrientation::Vertical looping=true>
                        <CarouselContent class="h-40">
                            <CarouselItem>
                                {slide("Top", "bg-primary/15 text-primary")}
                            </CarouselItem>
                            <CarouselItem>
                                {slide("Middle", "bg-accent text-accent-foreground")}
                            </CarouselItem>
                            <CarouselItem>
                                {slide("Bottom", "bg-muted text-muted-foreground")}
                            </CarouselItem>
                        </CarouselContent>
                        <CarouselPrevious />
                        <CarouselNext />
                        <CarouselIndicator />
                    </Carousel>
                </div>
            </Demo>
        </Section>
    }
}

// ---------------------------------------------------------------------------
// CardCarousel (scroll-snap track + delegated controller hook)
// ---------------------------------------------------------------------------

#[component]
fn CardCarouselSection() -> impl IntoView {
    let tint = |hex: &str| {
        format!(
            "data:image/svg+xml;utf8,%3Csvg xmlns='http://www.w3.org/2000/svg' width='320' height='320'%3E%3Crect width='320' height='320' fill='%23{hex}'/%3E%3C/svg%3E"
        )
    };

    view! {
        <Section
            title="Card carousel"
            description="Scroll-snapping image track driven by use_card_carousel: hover reveals the nav, dots track the active slide."
        >
            <Demo>
                <CardCarousel>
                    <CardCarouselTrack>
                        <CardCarouselSlide>
                            <img class="object-cover size-full" src=tint("6366f1") />
                        </CardCarouselSlide>
                        <CardCarouselSlide>
                            <img class="object-cover size-full" src=tint("0ea5e9") />
                        </CardCarouselSlide>
                        <CardCarouselSlide>
                            <img class="object-cover size-full" src=tint("10b981") />
                        </CardCarouselSlide>
                    </CardCarouselTrack>
                    <CardCarouselOverlay>
                        <CardCarouselNav>
                            <CardCarouselNavButton attr:aria-label="Previous">
                                <Icon icon=icondata::LuChevronLeft />
                            </CardCarouselNavButton>
                            <CardCarouselNavButton attr:aria-label="Next">
                                <Icon icon=icondata::LuChevronRight />
                            </CardCarouselNavButton>
                        </CardCarouselNav>
                        <CardCarouselIndicators>
                            <CardCarouselIndicator attr:aria-current="true" />
                            <CardCarouselIndicator />
                            <CardCarouselIndicator />
                        </CardCarouselIndicators>
                    </CardCarouselOverlay>
                </CardCarousel>
            </Demo>
        </Section>
    }
}

// ---------------------------------------------------------------------------
// DataGrid — full trait-based, sortable / pinnable / hideable / editable /
// virtualized People grid.
// ---------------------------------------------------------------------------

/// Sortable, pinnable, hideable column set for the [`Person`] grid. `strum`
/// derives `Display` (header labels) and `EnumIter` (the iteration the grid
/// trait bound requires); the enum order is the column order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumIter)]
enum Column {
    Name,
    Email,
    Role,
}

impl AsRef<str> for Column {
    fn as_ref(&self) -> &str {
        match self {
            Self::Name => "Name",
            Self::Email => "Email",
            Self::Role => "Role",
        }
    }
}

impl PinnableColumn for Column {
    fn pinnable_columns() -> &'static [(Self, i32)] {
        &[(Self::Name, 220), (Self::Email, 280), (Self::Role, 160)]
    }
}

impl DataGridColumn for Column {
    fn colindex(self) -> i32 {
        self as i32 + 2
    }
}

impl SortableColumn<Person> for Column {
    fn compare(self, a: &Person, b: &Person) -> Option<std::cmp::Ordering> {
        match self {
            Self::Name => Some(a.name.cmp(&b.name)),
            Self::Email => Some(a.email.cmp(&b.email)),
            Self::Role => Some(a.role.cmp(&b.role)),
        }
    }
}

/// One row of the People grid.
#[derive(Debug, Clone, Default)]
struct Person {
    id: i32,
    name: String,
    email: String,
    role: String,
}

impl DataGridRow for Person {
    type Id = i32;
    type Column = Column;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn matches_filter(&self, filter: &str) -> bool {
        let f = filter.to_lowercase();
        self.name.to_lowercase().contains(&f) || self.email.to_lowercase().contains(&f)
    }

    fn get_value(&self, col: Self::Column) -> String {
        match col {
            Column::Name => self.name.clone(),
            Column::Email => self.email.clone(),
            Column::Role => self.role.clone(),
        }
    }

    fn render_cell(&self, col: Self::Column) -> AnyView {
        match col {
            Column::Role => {
                let variant = match self.role.as_str() {
                    "Admin" => BadgeVariant::Default,
                    "Manager" => BadgeVariant::Accent,
                    _ => BadgeVariant::Muted,
                };
                let role = self.role.clone();
                view! { <Badge variant=variant>{role}</Badge> }.into_any()
            }
            other => view! { <span class="line-clamp-1">{self.get_value(other)}</span> }.into_any(),
        }
    }
}

/// Computed once: per-column CSS size variables consumed by the grid cells.
static GRID_STYLE: LazyLock<String> = LazyLock::new(generate_grid_style::<Column>);

fn sample_people() -> Vec<Person> {
    let first = [
        "Aria", "Bram", "Cleo", "Dimi", "Esme", "Finn", "Gaia", "Hugo", "Ivy", "Joss",
    ];
    let last = ["Vale", "Ostrom", "Quil", "Renn", "Sable"];
    let roles = ["Admin", "Manager", "Member"];
    (0usize..120)
        .map(|i| {
            let name = format!("{} {}", first[i % first.len()], last[i % last.len()]);
            let email = format!(
                "{}.{i}@openworkspace.dev",
                first[i % first.len()].to_lowercase(),
            );
            Person {
                id: i32::try_from(i).unwrap_or_default(),
                name,
                email,
                role: roles[i % roles.len()].to_string(),
            }
        })
        .collect()
}

#[component]
fn DataGridSection() -> impl IntoView {
    let people = RwSignal::new(sample_people());

    // View state shared with GenericGridHeader.
    let sort_signals: HashMap<Column, RwSignal<SortDirection>> = Column::iter()
        .map(|c| (c, RwSignal::new(SortDirection::None)))
        .collect();
    let sort_signals = StoredValue::new(sort_signals);
    let pinned: RwSignal<HashSet<Column>> = RwSignal::new(HashSet::new());
    let visible: RwSignal<HashSet<String>> =
        RwSignal::new(Column::iter().map(|c| c.as_ref().to_string()).collect());

    let selected: RwSignal<HashSet<i32>> = RwSignal::new(HashSet::new());

    // In-place editing context, scoped to this grid's Column type. Provided here
    // so every EditableCellContent below resolves the same instance via context.
    use_cell_edit::<Column>();
    let on_save = Callback::new(move |(row_idx, col, value): (usize, Column, String)| {
        people.update(|rows| {
            if let Some(p) = rows.get_mut(row_idx) {
                match col {
                    Column::Name => p.name = value,
                    Column::Email => p.email = value,
                    Column::Role => p.role = value,
                }
            }
        });
    });

    // Re-sort the underlying data whenever any column's sort direction changes.
    Effect::new(move |_| {
        let active = sort_signals.with_value(|signals| {
            signals.iter().find_map(|(col, sig)| {
                let dir = sig.get();
                (dir != SortDirection::None).then_some((*col, dir))
            })
        });
        if let Some((col, dir)) = active {
            people.update(|rows| col.sort_rows(rows, dir));
        }
    });

    let total_rows = Signal::derive(move || people.with(Vec::len));
    let rowcount = Signal::derive(move || i32::try_from(people.with(Vec::len)).unwrap_or(i32::MAX));

    let row_count_signal = Signal::derive(move || people.with(Vec::len));
    let selected_count_signal = Signal::derive(move || selected.with(HashSet::len));
    let handle_select_all = Callback::new(move |checked: bool| {
        if checked {
            let all: HashSet<i32> = people.with(|rows| rows.iter().map(Person::id).collect());
            selected.set(all);
        } else {
            selected.update(HashSet::clear);
        }
    });

    let data = Signal::derive(move || people.get());

    view! {
        <Section
            title="Data grid"
            description="The virtualized, generic data grid: sort and pin columns from the header menu, hide a column, double-click a cell to edit, and scroll 120 rows with only the visible window rendered."
        >
            <Demo col=true>
                <DataGridToolbar>
                    <span class="text-sm font-medium">"People"</span>
                    <span class="text-sm text-muted-foreground">
                        {move || format!("{} selected", selected_count_signal.get())}
                    </span>
                </DataGridToolbar>

                <VirtualizedGrid
                    total_rows=total_rows
                    rowcount=rowcount
                    colcount=4
                    style=GRID_STYLE.as_str()
                    class="h-[420px]"
                >
                    <GenericGridHeader
                        row_count_signal=row_count_signal
                        selected_count_signal=selected_count_signal
                        handle_select_all=handle_select_all
                        sort_signals=sort_signals
                        pinned_columns_signal=pinned
                        visible_columns_signal=visible
                    />
                    <VirtualizedGridBody>
                        <VirtualFor
                            data=data
                            key=|row: &Person| row.id
                            children=move |idx, row| {
                                view! {
                                    <PersonRow
                                        idx=idx
                                        row=row
                                        pinned=pinned
                                        visible=visible
                                        selected=selected
                                        on_save=on_save
                                    />
                                }
                            }
                        />
                    </VirtualizedGridBody>
                </VirtualizedGrid>
            </Demo>
        </Section>
    }
}

/// One rendered grid row: the sticky select cell plus the pinned and non-pinned
/// data cells, each editable in place.
#[expect(
    clippy::implicit_hasher,
    reason = "props bind the grid's concrete HashSet state signals; a #[component] cannot be generic over the hasher"
)]
#[component]
fn PersonRow(
    idx: usize,
    row: Person,
    pinned: RwSignal<HashSet<Column>>,
    visible: RwSignal<HashSet<String>>,
    selected: RwSignal<HashSet<i32>>,
    on_save: Callback<(usize, Column, String)>,
) -> impl IntoView {
    let row_id = row.id;
    let is_selected = Signal::derive(move || selected.with(|s| s.contains(&row_id)));
    let toggle = Callback::new(move |checked: bool| {
        selected.update(|s| {
            if checked {
                s.insert(row_id);
            } else {
                s.remove(&row_id);
            }
        });
    });

    // Sticky pinned cells: GridPinnedCell owns the left offset and z-index.
    let pinned_cells = {
        let row = row.clone();
        move || {
            Column::iter()
                .filter(|c| pinned.with(|p| p.contains(c)))
                .map(|col| {
                    let value = row.get_value(col);
                    view! {
                        <GridPinnedCell col=col pinned_columns_signal=pinned>
                            <div class="py-1.5 px-2 size-full">
                                <EditableCellContent
                                    row_idx=idx
                                    col=col
                                    value=value
                                    on_save=on_save
                                />
                            </div>
                        </GridPinnedCell>
                    }
                })
                .collect_view()
        }
    };

    // Flowing cells: hidden when pinned (rendered sticky above) or toggled off.
    let body_cells = move || {
        Column::iter()
            .map(|col| {
                let value = row.get_value(col);
                view! {
                    <GridCell
                        colindex=col.colindex()
                        column=col.css_safe_name()
                        visible=col.is_visible(pinned, visible)
                    >
                        <div class="py-1.5 px-2 size-full">
                            <EditableCellContent row_idx=idx col=col value=value on_save=on_save />
                        </div>
                    </GridCell>
                }
            })
            .collect_view()
    };

    view! {
        <GridRow rowindex=idx + 2 index=idx>
            <GridSelectCell>
                <div class="py-2.5 px-3">
                    <Checkbox
                        checked=is_selected
                        on_checked_change=toggle
                        aria_label="Select row"
                    />
                </div>
            </GridSelectCell>
            {pinned_cells}
            {body_cells}
        </GridRow>
    }
}
