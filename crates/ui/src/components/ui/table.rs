use crate::clx;
use leptos::prelude::*;

clx! {Table, table, "w-full max-w-7xl text-sm caption-bottom"}
clx! {TableHeader, thead, "[&_tr]:border-b sticky top-0 z-10 bg-card"}
clx! {TableBody, tbody, "[&_tr:last-child]:border-0"}
clx! {TableRow, tr, "border-b transition-colors data-[state=selected]:bg-muted hover:bg-muted/50"}
clx! {TableHead, th, "h-10 px-2 text-left align-middle font-medium text-muted-foreground [&:has([role=checkbox])]:pr-0 [&>[role=checkbox]]:translate-y-[2px]"}
clx! {TableCell, td, "p-4 align-middle [&:has([role=checkbox])]:pr-0 [&:has([role=checkbox])]:pl-3"}
clx! {TableCaption, caption, "mt-4 text-sm text-muted-foreground"}
