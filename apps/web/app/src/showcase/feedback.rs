//! Alerts, progress, spinners, skeletons, empty states.

use leptos::prelude::*;
use ui::{
    Alert, AlertDescription, AlertTitle, AlertVariant, Empty, EmptyDescription, EmptyHeader,
    EmptyMedia, EmptyTitle, Progress, Skeleton, Spinner,
};

use super::{Demo, PageShell};

/// Alerts, progress, spinners, skeletons, empty states.
#[component]
pub fn FeedbackPage() -> impl IntoView {
    view! {
        <PageShell title="Feedback" subtitle="Status, progress, and empty states.">
            <Demo title="Alert">
                <div class="flex w-full flex-col gap-3">
                    <Alert>
                        <AlertTitle>"Heads up"</AlertTitle>
                        <AlertDescription>"Your booking is confirmed."</AlertDescription>
                    </Alert>
                    <Alert variant=AlertVariant::Destructive>
                        <AlertTitle>"Conflict"</AlertTitle>
                        <AlertDescription>"That desk is already booked."</AlertDescription>
                    </Alert>
                </div>
            </Demo>
            <Demo title="Progress / Spinner">
                <Progress value=66.0 class="w-full" />
                <Spinner />
            </Demo>
            <Demo title="Skeleton">
                <div class="flex w-full flex-col gap-2">
                    <Skeleton class="h-8 w-full rounded-md" />
                    <Skeleton class="h-4 w-3/4 rounded-md" />
                </div>
            </Demo>
            <Demo title="Empty">
                <Empty class="py-6">
                    <EmptyHeader>
                        <EmptyMedia>"📭"</EmptyMedia>
                        <EmptyTitle>"No bookings"</EmptyTitle>
                        <EmptyDescription>"You have nothing booked today."</EmptyDescription>
                    </EmptyHeader>
                </Empty>
            </Demo>
        </PageShell>
    }
}
