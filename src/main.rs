pub mod config;
pub mod shapes;
pub mod state;
pub mod ui;
use core::time;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleRate};
use native_windows_gui as nwg;
use spectrum_analyzer::scaling::divide_by_N_sqrt;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use ui::UI;

// // standard std exposing io methods
use ggez::glam::*;
use ggez::graphics::{self, Color, DrawMode, FillOptions, Text};
use ggez::{event, ContextBuilder};
use ggez::{Context, GameResult};

use crate::config::{GlobalConfiguration, UpdateStatus};
use crate::shapes::{spiraling, Shape};
fn main() -> GameResult {
    let mut global_config = GlobalConfiguration::default();
    let status = if global_config.configuration.exists() {
        match global_config.configuration.status() {
            Ok(status) => match status {
                UpdateStatus::UpToDate => global_config.configuration.retrieve_from_registry(),
                // DataStatus::NewerAlreadyInstalled => config.retrieve_from_registry(),
                _ => todo!(),
            },
            Err(err) => {
                println!("Error accessing the config : {} ", err);
                global_config.configuration.update_to_registry()
            }
        }
    } else {
        global_config.configuration.update_to_registry()
    };

    if status.is_err() {
        eprintln!("An error occured retrieving the configuration...");
        println!("error : {}", status.as_ref().unwrap_err());
        // todo!()
    }
    drop(status);
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range
        .next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    let sample_rate = supported_config.sample_rate();
    let config = supported_config;
    let r = DataCollected { points: vec![] };
    let clone_data = r.clone();
    let m = Mutex::new(clone_data);
    let reader = Arc::new(m);
    let clone = reader.clone();
    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };
    println!(
        "The sampling format for the configuration is {:?}.",
        config.sample_format()
    );
    let stream = match config.sample_format() {
        cpal::SampleFormat::U8 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<u8, u8>(data, &clone),
            err_fn,
            None,
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<u16, u16>(data, &clone),
            err_fn,
            None,
        ),

        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i16, i16>(data, &clone),
            err_fn,
            None,
        ),
        cpal::SampleFormat::I32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<i32, f32>(data, &clone),
            err_fn,
            None,
        ),
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<f32, f32>(data, &clone),
            err_fn,
            None,
        ),
        // _ => todo!(),
        sample_format => {
            println!(
                "An error occured trying to parse the sample format {}",
                sample_format
            );
            sleep(time::Duration::from_secs(15));
            panic!("Unsupported sample format {sample_format}")
        }
    }
    .unwrap();
    stream.play().unwrap();

    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let cb = ContextBuilder::new("Materialize", "Dekharen").window_mode(
        ggez::conf::WindowMode {
            transparent: false,
            visible: true,
            resize_on_scale_factor_change: true,
            logical_size: None,
            width: global_config.configuration.screen_size.value.0,
            height: global_config.configuration.screen_size.value.1,
            maximized: false,
            fullscreen_type: ggez::conf::FullscreenType::Windowed,
            borderless: true,
            min_width: 1.0,
            max_width: 0.0,
            min_height: 1.0,
            max_height: 0.0,
            resizable: false,
        }, // .dimensions(screen_size.0, screen_size.1),
    );
    let (mut ctx, event_loop) = cb.build()?; // `?` because the build function may fail
    ctx.gfx.set_window_title("Materialize");
    let state = MainState::new(
        &mut ctx,
        sample_rate,
        vec![vec![0.0], vec![0.0], vec![0.0], vec![0.0]],
        global_config,
        Arc::clone(&reader),
    );
    event::run(ctx, event_loop, state);
}

fn write_input_data<T, U>(input: &[T], reader: &Arc<Mutex<DataCollected>>)
where
    DataCollected: Reader<T, U>,
    T: Sample,
    U: Sample + cpal::FromSample<T> + std::fmt::Debug,
    f32: From<U>,
{
    let r = Arc::clone(&reader);
    let mut guard = r.lock().unwrap();
    guard.add(input);
}

// TODO type archmutex data ?
struct MainState {
    g_config: GlobalConfiguration,
    values: Vec<Vec<f32>>,
    reader: Arc<Mutex<DataCollected>>,
    sample_rate: SampleRate,
    counter: Vec<(f32, f32, f32)>,
    pos_x: f32,
    ui: UI,
}
impl MainState {
    fn new(
        ctx: &mut Context,
        sample_rate: SampleRate,
        values: Vec<Vec<f32>>,
        g_config: GlobalConfiguration,
        reader: Arc<Mutex<DataCollected>>,
    ) -> Self {
        let number_of_items = g_config.configuration.number_of_items.value;
        MainState {
            ui: UI::new(ctx),
            g_config,
            sample_rate,
            reader,
            values,
            counter: vec![(1.0, 1.0, 1.0); number_of_items as usize],
            pos_x: 0_f32,
        }
    }

    fn sample(&mut self) {
        let mut guard = self.reader.lock().unwrap();
        if guard.points.len() < self.g_config.configuration.size_arr.value {
            return;
        }
        let pts = guard.points.clone();
        let windowed_values = hann_window(&pts[..self.g_config.configuration.size_arr.value]);
        let (_, remain) = pts.split_at(self.g_config.configuration.size_arr.value / 2);
        guard.points = remain.to_vec();
        let spectrum_frequencies = samples_fft_to_spectrum(
            &windowed_values,
            self.sample_rate.0,
            FrequencyLimit::Max(self.g_config.configuration.polled_frequencies.value as f32),
            // Recommended scaling/normalization by `rustfft`.
            Some(&divide_by_N_sqrt),
        )
        .unwrap();
        let spectrum_map = spectrum_frequencies.to_map();
        let vector_spectrum: Vec<Vec<f32>> = self.map_to_representation(&spectrum_map);
        self.values = vector_spectrum;
    }
    fn map_to_representation(&self, map: &std::collections::BTreeMap<u32, f32>) -> Vec<Vec<f32>> {
        let len = &self.g_config.configuration.number_of_items.value;
        let mut index = 0;
        let usize_len = *len as usize;
        let mut vector = vec![vec![0.0]; usize_len];
        let chunk_s = self.g_config.configuration.viewed_frequencies.value / len;

        for (&key, value) in map.iter() {
            if key >= (index + 1) as u32 * chunk_s {
                if index < usize_len - 1 {
                    index += 1;
                } else {
                    // Early return; we only use half of the total frequencies for viewing, to identify
                    // the changes better.
                    break;
                }
            }
            vector[index].push(*value)
        }
        return vector;
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn text_input_event(
        &mut self,
        _ctx: &mut Context,
        character: char,
    ) -> Result<(), ggez::GameError> {
        self.ui.gui.input.text_input_event(character);
        Ok(())
    }
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.pos_x = self.pos_x + 2.0;
        if !self.g_config.open {
            self.ui.update_menu(ctx, &mut self.g_config)?;
        }
        // let win = ctx.gfx.window();
        // win.set_inner_size(PhysicalSize::new(200, 200));
        Ok(())
    }
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: ggez::input::keyboard::KeyInput,
        _repeated: bool,
    ) -> Result<(), ggez::GameError> {
        match input.keycode {
            Some(key) => match key {
                ggez::input::keyboard::KeyCode::Space => {
                    self.g_config.open = !self.g_config.open;
                    Ok(())
                }
                ggez::input::keyboard::KeyCode::Escape => {
                    ctx.request_quit();
                    Ok(())
                }
                _ => Ok(()),
            },
            _ => Ok(()),
        }
    }

    // TODO Move sampling to updating logic
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::from([0.0, 0.0, 0.0, 1.0]));
        // canvas.set_scissor_rect(graphics::Rect {
        //     x: 1f32,
        //     y: 1f32,
        //     w: 1f32,
        //     h: 1f32,
        // }).ok();
        // ctx.gfx.size();
        let mode = DrawMode::Fill(FillOptions::DEFAULT);
        let mut x_position = 50_f32;
        const Y_POSITION: f32 = 1000_f32;

        self.sample();
        let mut i = 0;
        //  let spiral_iter = spiral::ManhattanIterator::new(self.configuration.screen_size.0/2, self.configuration.screen_size.1/2, );
        let config = &self.g_config.configuration;
        if !self.g_config.open {
            let version = String::from("v.") + &*config.version.value;
            let vers = Text::new(version);
            canvas.draw(&vers, [0.0, 0.0]);
        }
        if self.counter.len() < config.number_of_items.value as usize {
            let mut fill = vec![
                (1.0f32, 1.0f32, 1.0f32);
                config.number_of_items.value as usize - self.counter.len()
            ];
            self.counter.append(&mut fill);
            drop(fill);
        } else if self.counter.len() > config.number_of_items.value as usize {
            self.counter.truncate(config.number_of_items.value as usize);
            self.values.truncate(config.number_of_items.value as usize);
            //    self.counter = c;
        }
        let shape = shapes::ShapeBuilder::new(config.kind.clone());
        let spiral_values = spiraling(self.values.len());
        let mut spiral = spiral_values.iter();
        for frequency in &self.values {
            // if let Some((x, y)) = spiral.next() {}
            let default_pos = (0.0_f32, 0.0_f32);
            let (x, y) = spiral.next().unwrap_or(&default_pos);
            let iter = frequency.iter();
            let mut sum = iter.fold(0_f32, |x, &x2| x + x2);
            sum = sum * config.scale.value;

            if sum > config.screen_size.value.1 - 50.0 {
                sum = config.screen_size.value.1 - 50.0
            }
            self.counter[i].0 = sum;

            // Positioning with an exponential animation position = position + (destination -
            // position) * speed * dt
            //
            // Main bar (no history)
            self.counter[i].1 += (self.counter[i].0 - self.counter[i].1) * 50.0 * (1.0 / 125.0);
            // Afterimage (history, slower animation)
            self.counter[i].2 += (self.counter[i].0 - self.counter[i].2) * 50.0 * (1.0 / 250.0);
            // if self.counter[i].2 < self.counter[i].1 {
            //     self.counter[i].2 = self.counter[i].1
            // }

            // TODO
            // remove magic numbers (Speed & dt)
            let color = match i % 4 {
                0 => Color::WHITE,
                1 => Color::CYAN,
                2 => Color::RED,
                _ => Color::YELLOW,
            };

            match shape.clone() {
                Shape::Cyclic(shape) => shape.draw(
                    ctx,
                    mode,
                    color,
                    (
                        config.screen_size.value.0 / 2.0 + x,
                        config.screen_size.value.1 / 2.0 + y,
                    ),
                    (self.counter[i].1, self.counter[i].2),
                    &mut canvas,
                ),
                Shape::RoundedRectangular(shape) => shape.draw(
                    ctx,
                    mode,
                    color,
                    (x_position, Y_POSITION),
                    (self.counter[i].1, self.counter[i].2),
                    20.0,
                    &mut canvas,
                ),
            };
            // let rectangle = graphics::Mesh::new_rounded_rectangle(
            //     ctx,
            //     mode,
            //     graphics::Rect::new(x_position, Y_POSITION, 10_f32, self.counter[i].1),
            //     20_f32,
            //     color,
            // )
            // .unwrap();
            // color.a = 0.1;
            // let after_image = graphics::Mesh::new_rounded_rectangle(
            //     ctx,
            //     mode,
            //     graphics::Rect::new(x_position, Y_POSITION, 10.50_f32, self.counter[i].2),
            //     20_f32,
            //     color,
            // )
            // .unwrap();
            // canvas.draw(&rectangle, Vec2::new(0_f32, -self.counter[i].1));

            // canvas.draw(&after_image, Vec2::new(0_f32, -self.counter[i].2)); //TODO this needs to
            //                                                                  //be stored & updated every frame, instead of only appearing once
            i += 1;
            x_position += 50_f32;
        }
        // for &val in self.values {
        // let circle = graphics::Mesh::new_circle(
        //     ctx,
        //     graphics::DrawMode::fill(),
        //     Vec2::new(0.0, 0.0),
        //     100.0,
        //     2.0,
        //     Color::WHITE,
        // )?;
        // canvas.draw(&pt, Vec2::new(self.pos_x, 380.0));
        // }
        if !self.g_config.open {
            self.ui.draw_ui(&mut canvas)?;
        }
        canvas.finish(ctx)?;
        Ok(())
    }
}
// struct Grid {
//     width: usize,
//     height: usize,
//     cells: Vec<Cell>,
// }
// impl Grid {
//     // Width and height of the Grid
//     pub fn new(width: usize, height: usize) -> Self {
//         return Self {
//             width,
//             height,
//             cells: vec![Cell::new(false); width * height],
//         };
//     }
// }

// #[derive(Clone, Debug)]
// pub struct Cell {
//     alive: bool,
// }

// impl Cell {
//     pub fn new(alive: bool) -> Self {
//         return Self { alive };
//     }
// }

#[derive(Debug, Clone)]
struct DataCollected {
    // results_total: f32,
    points: Vec<f32>,
}

trait Reader<X, A> {
    // fn read_sample<T, U>(&self, sample: U)
    // where
    //     T: Sample,
    //     U: Sample + cpal::FromSample<T> + std::fmt::Debug;
    fn add(&mut self, samples: &[X]);
}

impl Reader<u8, u8> for DataCollected {
    // fn read_sample<T, U>(&self, sample: U)
    // where
    //     T: Sample,
    //     U: Sample + cpal::FromSample<T> + std::fmt::Debug,
    // {
    //     print!("a");
    // }

    fn add(&mut self, samples: &[u8]) {
        self.points.extend(samples.iter().map(|y| {
            let s: f32 = y.to_sample();
            return s; //100.0;
                      // return *y as f32 - 128.0; // * 10 ?
        }));
    }
}

impl Reader<u16, u16> for DataCollected {
    // fn read_sample<T, U>(&self, sample: U)
    // where
    //     T: Sample,
    //     U: Sample + cpal::FromSample<T> + std::fmt::Debug,
    // {
    //     print!("a");
    // }

    fn add(&mut self, samples: &[u16]) {
        self.points.extend(samples.iter().map(|y| {
            let s: f32 = y.to_sample();
            return s;
        }));
    }
}
impl Reader<i16, i16> for DataCollected {
    // fn read_sample<T, U>(&self, sample: U)
    // where
    //     T: Sample,
    //     U: Sample + cpal::FromSample<T> + std::fmt::Debug,
    // {
    //     print!("a");
    // }

    fn add(&mut self, samples: &[i16]) {
        self.points.extend(samples.iter().map(|y| {
            let s: f32 = y.to_sample();
            return s;
        }));
    }
}
impl Reader<i32, f32> for DataCollected {
    // fn read_sample<T, U>(&self, sample: U)
    // where
    //     T: Sample,
    //     U: Sample + cpal::FromSample<T> + std::fmt::Debug,
    // {
    //     print!("a");
    // }

    fn add(&mut self, samples: &[i32]) {
        self.points.extend(samples.iter().map(|y| {
            let s: f32 = y.to_sample();
            return s * 100.0;
        }));
    }
}
impl Reader<f32, f32> for DataCollected {
    // fn read_sample<T, U>(&self, sample: U)
    // where
    //     T: Sample,
    //     U: Sample + cpal::FromSample<T> + std::fmt::Debug,
    // {
    //     print!("a");
    // }

    fn add(&mut self, samples: &[f32]) {
        self.points.extend(samples.iter().map(|y| {
            return *y; // * 10 ?
        }));
    }
}
// #[derive(Debug, Clone)]
// struct Display {
//     name: String,
// }
// impl Display {
//     fn new() -> Display {
//         return Display {
//             name: String::from("console"),
//         };
//     }
// }

/// Config for the start of the game
#[derive(Debug, Clone)]
pub struct Config {
    pub grid_width: usize,
    pub grid_height: usize,
    pub cell_size: f32,
    pub screen_size: (f32, f32),
    pub fps: u32,
    pub terminal_display: bool,
}
