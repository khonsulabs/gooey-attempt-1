use gooey_core::{
    figures::{Figure, Point, Rectlike, Size, SizedRect, Vector, Vectorlike},
    Scaled, Transmogrifier, TransmogrifierContext,
};
use gooey_rasterizer::{ContentArea, Rasterizer, Renderer, WidgetRasterizer};

use super::LayoutChild;
use crate::layout::{Layout, LayoutTransmogrifier};

impl<R: Renderer> Transmogrifier<Rasterizer<R>> for LayoutTransmogrifier {
    type State = ();
    type Widget = Layout;

    fn receive_command(
        &self,
        _command: <Self::Widget as gooey_core::Widget>::Command,
        context: &mut TransmogrifierContext<'_, Self, Rasterizer<R>>,
    ) {
        context.frontend.set_needs_redraw();
    }
}

impl<R: Renderer> WidgetRasterizer<R> for LayoutTransmogrifier {
    fn render(
        &self,
        context: &mut TransmogrifierContext<'_, Self, Rasterizer<R>>,
        content_area: &ContentArea,
    ) {
        let bounds = content_area.bounds();
        for_each_measured_widget(context, bounds.size(), |layout, child_bounds| {
            context.frontend.with_transmogrifier(
                layout.registration.id(),
                |transmogrifier, mut child_context| {
                    transmogrifier.render_within(
                        &mut child_context,
                        child_bounds.translate(content_area.location).as_rect(),
                        context.style,
                    );
                },
            );
        });
    }

    fn measure_content(
        &self,
        context: &mut TransmogrifierContext<'_, Self, Rasterizer<R>>,
        constraints: Size<Option<f32>, Scaled>,
    ) -> Size<f32, Scaled> {
        let mut extents = Vector::default();
        let context_size = context.frontend.renderer().unwrap().size();
        let constrained_size = Size::new(
            constraints.width.unwrap_or(context_size.width),
            constraints.height.unwrap_or(context_size.height),
        );
        for_each_measured_widget(context, constrained_size, |_layout, child_bounds| {
            extents = extents.max(&child_bounds.as_extents().extent.to_vector());
        });
        extents.to_size()
    }
}

#[allow(clippy::cast_precision_loss)]
fn for_each_measured_widget<R: Renderer, F: FnMut(&LayoutChild, SizedRect<f32, Scaled>)>(
    context: &TransmogrifierContext<'_, LayoutTransmogrifier, Rasterizer<R>>,
    constraints: Size<f32, Scaled>,
    mut callback: F,
) {
    for child in context.widget.children.layout_children() {
        let layout_surround = child.layout.surround_in_points(constraints);
        let layout_max_size = child
            .layout
            .size_in_points(constraints)
            .min(&(constraints - layout_surround.minimum_size()));

        // Constrain the child to whatever remains after the left/right/top/bottom
        // measurements
        let child_constraints =
            Size::new(Some(layout_max_size.width), Some(layout_max_size.height));

        // Ask the child to measure
        let child_size = context
            .frontend
            .with_transmogrifier(
                child.registration.id(),
                |transmogrifier, mut child_context| {
                    transmogrifier
                        .content_size(&mut child_context, child_constraints)
                        .total_size()
                },
            )
            .expect("unknown transmogrifier");

        // If the layout has an explicit width/height, we should return it if it's a
        // value larger than what the child reported
        let child_size = child_size.max(&layout_max_size);
        let remaining_size = constraints - child_size;
        // If either top or left are Auto, we look at bottom or right,
        // respectively. If the corresponding dimension is also Auto, we divide
        // the remaining amount in two. If the other dimension is specified,
        // however, we subtract its measurement from the remaining value and use
        // it as top/left.
        let child_left = layout_surround.left.unwrap_or_else(|| {
            let remaining_width = Figure::new(remaining_size.width);
            layout_surround
                .right
                .map_or_else(|| remaining_width / 2., |right| remaining_width - right)
        });
        let child_top = layout_surround.top.unwrap_or_else(|| {
            let remaining_height = Figure::new(remaining_size.height);
            layout_surround
                .bottom
                .map_or_else(|| remaining_height / 2., |bottom| remaining_height - bottom)
        });
        callback(
            &child,
            SizedRect::new(Point::from_figures(child_left, child_top), child_size),
        );
    }
}
