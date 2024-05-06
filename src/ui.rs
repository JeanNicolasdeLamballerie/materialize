use ggegui::{egui, Gui};
use ggez::{graphics::DrawParam, Context, GameResult};
pub struct UI {
    pub gui: Gui,
}
impl UI {
    pub fn new(ctx: &mut Context) -> Self {
        Self { gui: Gui::new(ctx) }
    }
    pub fn update_menu(
        &mut self,
        ctx: &mut Context,
        config: &mut crate::config::Configuration,
    ) -> GameResult {
        let gui_ctx = self.gui.ctx();
        let options = egui::Window::new("Options");
        // options = options.min_size([800.0_f32, 800.0_f32]);
        options
            .default_size([800.0_f32, 800.0_f32])
            .resizable(false)
            .show(&gui_ctx, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());
                let label = ui.label("Close button");

                let mut root = config.size_arr.value.ilog2(); //.nth_root(9);
                let initial_root = root;
                ui.add(
                    egui::Slider::new(&mut root, 9..=14)
                        .text("size")
                        .custom_formatter(|v, _| (2_usize.pow(v as u32)).to_string()),
                );

                if root != initial_root {
                    config.size_arr.value = 2usize.pow(root);
                }
                // ui.style_mut();
                //

                ui.add(
                    egui::Slider::new(&mut config.scale.value, 0.0..=1000.0).text("scale"), // .custom_formatter(|v, _| (2_usize.pow(v as u32)).to_string()),
                );

                ui.add(
                    egui::Slider::new(&mut config.number_of_items.value, 1..=500)
                        .text("Number of splits in the frequencies")
                        .step_by(1f64), // .custom_formatter(|v, _| (2_usize.pow(v as u32)).to_string()),
                );
                ui.add(
                    egui::Slider::new(
                        &mut config.viewed_frequencies.value,
                        1..=config.polled_frequencies.value,
                    )
                    .step_by(1f64)
                    .text("Frequencies viewed (out of total frequencies). (High polled/low viewed = higher precision)"), // .custom_formatter(|v, _| (2_usize.pow(v as u32)).to_string()),
                );
                ui.add(
                    egui::Slider::new(&mut config.polled_frequencies.value, 50..=20000)
                        .step_by(1f64)
                        .text("Frequencies retained from the analysis (This should be as high as possible)"), // .custom_formatter(|v, _| (2_usize.pow(v as u32)).to_string()),
                );
                if ui.button("Close").labelled_by(label.id).clicked() {
                    println!("button clicked");
                    config.open = !config.open;
                }
            });
        self.gui.update(ctx);
        Ok(())
    }
    pub fn draw_ui(&mut self, canvas: &mut ggez::graphics::Canvas) -> GameResult {
        canvas.draw(&self.gui, DrawParam::default().dest(ggez::glam::Vec2::ZERO));
        Ok(())
    }
}
