//! Tables, lists, avatars, breadcrumbs, pagination, chart.

use leptos::prelude::*;
use ui::{
    Avatar, AvatarFallback, AvatarGroup, Breadcrumb, BreadcrumbItem, BreadcrumbLink,
    BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator, ChartContainer, ChartSeries, Item,
    ItemContent, ItemDescription, ItemGroup, ItemMedia, ItemTitle, Pagination, PaginationContent,
    PaginationItem, PaginationLink, PaginationNext, PaginationPrevious, Table, TableBody,
    TableCell, TableHead, TableHeader, TableRow,
};

use super::{Demo, PageShell};

/// Tables, lists, avatars, breadcrumbs, pagination, chart.
#[component]
#[expect(
    clippy::too_many_lines,
    reason = "a flat gallery page of independent demos"
)]
pub fn DataPage() -> impl IntoView {
    view! {
        <PageShell title="Data" subtitle="Tables, lists, identities, navigation.">
            <Demo title="Table">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>"Desk"</TableHead>
                            <TableHead>"Floor"</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        <TableRow>
                            <TableCell>"A-12"</TableCell>
                            <TableCell>"2"</TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell>"B-03"</TableCell>
                            <TableCell>"3"</TableCell>
                        </TableRow>
                    </TableBody>
                </Table>
            </Demo>
            <Demo title="Avatars + Item">
                <AvatarGroup>
                    <Avatar>
                        <AvatarFallback>"OW"</AvatarFallback>
                    </Avatar>
                    <Avatar>
                        <AvatarFallback>"RT"</AvatarFallback>
                    </Avatar>
                </AvatarGroup>
                <ItemGroup class="w-full">
                    <Item>
                        <ItemMedia>
                            <Avatar>
                                <AvatarFallback>"OW"</AvatarFallback>
                            </Avatar>
                        </ItemMedia>
                        <ItemContent>
                            <ItemTitle>"Window desk"</ItemTitle>
                            <ItemDescription>"Floor 2 · near the kitchen"</ItemDescription>
                        </ItemContent>
                    </Item>
                </ItemGroup>
            </Demo>
            <Demo title="Breadcrumb">
                <Breadcrumb>
                    <BreadcrumbList>
                        <BreadcrumbItem>
                            <BreadcrumbLink attr:href="#">"Campus"</BreadcrumbLink>
                        </BreadcrumbItem>
                        <BreadcrumbSeparator />
                        <BreadcrumbItem>
                            <BreadcrumbPage>"Floor 2"</BreadcrumbPage>
                        </BreadcrumbItem>
                    </BreadcrumbList>
                </Breadcrumb>
            </Demo>
            <Demo title="Pagination">
                <Pagination>
                    <PaginationContent>
                        <PaginationItem>
                            <PaginationPrevious attr:href="#" />
                        </PaginationItem>
                        <PaginationItem>
                            <PaginationLink attr:href="#">"1"</PaginationLink>
                        </PaginationItem>
                        <PaginationItem>
                            <PaginationLink is_active=true attr:href="#">
                                "2"
                            </PaginationLink>
                        </PaginationItem>
                        <PaginationItem>
                            <PaginationNext attr:href="#" />
                        </PaginationItem>
                    </PaginationContent>
                </Pagination>
            </Demo>
            <Demo title="Chart">
                <ChartContainer
                    id="bookings"
                    config=vec![
                        ChartSeries {
                            key: "bookings".into(),
                            label: "Bookings".into(),
                            color: "var(--chart-1)".into(),
                        },
                    ]
                    class="aspect-auto h-40 w-full"
                >
                    <svg viewBox="0 0 200 100" class="h-full w-full" preserveAspectRatio="none">
                        {[40_i32, 65, 50, 80, 60, 90]
                            .into_iter()
                            .enumerate()
                            .map(|(i, h)| {
                                let x = 12 + i32::try_from(i).unwrap_or(0) * 32;
                                view! {
                                    <rect
                                        x=x.to_string()
                                        y=(100 - h).to_string()
                                        width="20"
                                        height=h.to_string()
                                        rx="2"
                                        fill="var(--color-bookings)"
                                    ></rect>
                                }
                            })
                            .collect_view()}
                    </svg>
                </ChartContainer>
            </Demo>
        </PageShell>
    }
}
