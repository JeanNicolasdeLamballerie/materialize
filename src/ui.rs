use ggegui::{egui, Gui};
use ggez::{graphics::DrawParam, Context, GameResult};

use crate::{config::Configuration, shapes::ShapeKind};
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
        g_config: &mut crate::config::GlobalConfiguration,
    ) -> GameResult {
        let gui_ctx = self.gui.ctx();
        ctx.gfx.size();
        let options = egui::Window::new("Options");
        // options = options.min_size([800.0_f32, 800.0_f32]);
        options
            .default_size([
                g_config.configuration.screen_size.value.0 / 2.0,
                g_config.configuration.screen_size.value.1 / 2.0,
            ])
            .default_pos((
                g_config.configuration.screen_size.value.0 / 4.0,
                g_config.configuration.screen_size.value.1 / 4.0,
            ))
            .resizable(false)
            .show(&gui_ctx, |ui| {

                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());
                // let mut some_str = g_config.new_profile_name;
                let mut root = g_config.configuration.size_arr.value.ilog2(); //.nth_root(9);
                let initial_root = root;
               if ui.add(egui::Button::new("Select Profile")).clicked() {
                g_config.cfg_list_open = !g_config.cfg_list_open;
                };
                if g_config.cfg_list_open {
                    egui::SidePanel::right("profile_selector").show(&gui_ctx, |panel_ui| {
                     for  value in g_config.cfg_list.values().iter() {
                        panel_ui.add(egui::Button::new(value)); 
                     panel_ui.separator();
                        }
                     panel_ui.add(egui::TextEdit::singleline(&mut g_config.new_profile_name));
                        if panel_ui.add(egui::Button::new("Create")).clicked() {
                           let mut new_cfg = Configuration::default();
                            new_cfg.key = g_config.new_profile_name.clone();
                            g_config.new_profile_name = String::from("New Profile");
                            println!("Changed !");

                        }
                    });
                };
            let mut key_name : &str = &(String::from("Selected profile : ") + &g_config.configuration.key);
            ui.add(egui::TextEdit::singleline(&mut key_name));
            
               if ui.add(egui::Button::new("Select Shape")).clicked() {
                g_config.shape_list_open = !g_config.shape_list_open;
                };
                if g_config.shape_list_open {
                    egui::SidePanel::right("shape_selector").show(&gui_ctx, |panel_ui| {
                     for  value in ShapeKind::iterator() {
                       if panel_ui.add(egui::Button::new(value.to_str())).clicked(){
                                g_config.configuration.kind = value.clone();
                            }; 
                     panel_ui.separator();
                        }
                    });
                };
                // ui.add(egui::TextEdit::singleline(&mut g_config.new_profile_name).char_limit(50));
                ui.add(
                    egui::Slider::new(&mut root, 9..=14)
                        .text("Size of sample. The bigger the number, the less frequent the update (but the more accurate)")
                        .custom_formatter(|v, _| (2_usize.pow(v as u32)).to_string()),
                );

                if root != initial_root {
                    g_config.configuration.size_arr.value = 2usize.pow(root);
                }
                // ui.style_mut();
                //
                ui.separator();
                ui.add(
                    egui::Slider::new(&mut g_config.configuration.scale.value, 0.0..=1000.0).text("scale"), // .custom_formatter(|v, _| (2_usize.pow(v as u32)).to_string()),
                );

                ui.add(
                    egui::Slider::new(&mut g_config.configuration.number_of_items.value, 1..=500)
                        .text("Number of splits in the frequencies")
                        .step_by(1f64), // .custom_formatter(|v, _| (2_usize.pow(v as u32)).to_string()),
                );
                ui.separator();
                ui.add(
                    egui::Slider::new(
                        &mut g_config.configuration.viewed_frequencies.value,
                        1..=g_config.configuration.polled_frequencies.value,
                    )
                    .step_by(1f64)
                    .text("Frequencies viewed (out of total frequencies). (High polled/low viewed = higher precision)"), // .custom_formatter(|v, _| (2_usize.pow(v as u32)).to_string()),
                );
                ui.add(
                    egui::Slider::new(&mut g_config.configuration.polled_frequencies.value, 50..=20000)
                        .step_by(1f64)
                        .text("Frequencies retained from the analysis (This should be as high as possible)"), // .custom_formatter(|v, _| (2_usize.pow(v as u32)).to_string()),
                );
                ui.separator();

                let label = ui.label("Close button");
                if ui.button("Close").labelled_by(label.id).clicked() {
                    g_config.open = !g_config.open;
                }
            });
        self.gui.update(ctx);
        Ok(())
    }
    pub fn draw_ui(&mut self, canvas: &mut ggez::graphics::Canvas) -> GameResult {
        // println!("moments before disaster");
        canvas.draw(&self.gui, DrawParam::default().dest(ggez::glam::Vec2::ZERO));
        // println!("moments after disaster");
        Ok(())
    }
}
