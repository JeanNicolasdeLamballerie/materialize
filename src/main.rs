pub mod state;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BuildStreamError, Data, Sample, SampleFormat, SampleRate};
use ggez::mint::Point2;
use native_windows_gui as nwg;
use rustfft::num_complex::Complex32;
use rustfft::{num_complex::Complex, FftPlanner};
use spectrum_analyzer::scaling::divide_by_N_sqrt;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
use std::slice;
use std::sync::{Arc, Mutex, MutexGuard};
// use std::{cmp::Ordering, io};
// // standard std exposing io methods
// use rand::Rng;
use ggez::glam::*;
use ggez::graphics::{self, Color, DrawMode, FillOptions, StrokeOptions};
use ggez::{event, ContextBuilder};
use ggez::{Context, GameResult};

// struct MainState {
//     pos_x: f32,
// }

// impl MainState {
//     fn new() -> GameResult<MainState> {
//         let s = MainState { pos_x: 0.0 };
//         Ok(s)
//     }
// }
// use terminal_size::{terminal_size, Height, Width};

fn main() -> GameResult {
    let host = cpal::default_host();
    let mut device = host
        .default_output_device()
        .expect("no output device available");
    let devices = host.output_devices().expect("No devices available");
    // for possible_device in devices {
    //     if "Speakers (3- USB Audio Device)" == possible_device.name().unwrap() {
    //         device = possible_device;
    //         break;
    //     };
    // }
    println!("Main device : {}", device.name().unwrap());
    // std::thread::sleep(std::time::Duration::from_secs(1));
    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range
        .next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    let sample_rate = supported_config.sample_rate();
    let config = supported_config;
    println!("start");
    // const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/recorded.wav");
    // let spec = wav_spec_from_config(&config);
    // let writer = hound::WavWriter::create(PATH, spec)?;
    // let writer = Arc::new(Mutex::new(Some(writer)));

    // // A flag to indicate that recording is in progress.
    // println!("Begin recording...");
    let r = DataCollected {
        display: Display::new(),
        min: u8::MAX,
        max: u8::MIN,
        sample_count: 0,
        results_count: 0,
        results_total: 0_f32,
        points: vec![],
    };
    let clone_data = r.clone();
    let m = Mutex::new(clone_data);
    let reader = Arc::new(m);
    let clone = reader.clone();
    // let min = reader.min;
    // let max = reader.max;
    // // Run the input stream on a separate thread.

    // let reader = writer.clone();
    //
    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };
    println!("{:?}", config.sample_format());
    let stream = match config.sample_format() {
        cpal::SampleFormat::U8 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<u8, u8>(data, &clone),
            err_fn,
            None,
        ),
        // cpal::SampleFormat::I16 => device.build_input_stream(
        //     &config.into(),
        //     move |data, _: &_| write_input_data::<i16, i16>(data, &reader),
        //     err_fn,
        //     None,
        // ),
        // cpal::SampleFormat::I32 => device.build_input_stream(
        //     &config.into(),
        //     move |data, _: &_| write_input_data::<i32, i32>(data, &reader),
        //     err_fn,
        //     None,
        // ),
        // cpal::SampleFormat::F32 => device.build_input_stream(
        //     &config.into(),
        //     move |data, _: &_| write_input_data::<f32, f32>(data, &reader),
        //     err_fn,
        //     None,
        // ),
        _ => todo!(),
        // sample_format => Err(anyhow::Error::msg(format!(
        //     "Unsupported sample format '{sample_format}'"
        // ))),
    }
    .unwrap();
    stream.play().unwrap();

    // println!("start...");
    // std::thread::sleep(std::time::Duration::from_secs(3));
    // println!("start...");
    // drop(stream);
    // let r = reader.lock().unwrap();
    // let min = r.min;
    // let max = r.max;
    // let max = reader.max;
    // println!(
    //     "{} - {} || {} - {} ",
    //     min, max, r.results_count, r.sample_count
    // );
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    println!("{}", r.points.len());
    // let screen_size = (720., 720.);
    // let fps = 20;
    let cb = ContextBuilder::new("Materialize", "Dekharen").window_mode(
        ggez::conf::WindowMode {
            transparent: false,
            visible: true,
            resize_on_scale_factor_change: false,
            logical_size: None,
            width: 2000.0,
            height: 600.0,
            maximized: false,
            fullscreen_type: ggez::conf::FullscreenType::Windowed,
            borderless: false,
            min_width: 1999.0,
            max_width: 2001.0,
            min_height: 500.0,
            max_height: 1200.0,
            resizable: true,
        }, // .dimensions(screen_size.0, screen_size.1),
    );
    let (mut ctx, event_loop) = cb.build()?; // `?` because the build function may fail
    ctx.gfx.set_window_title("Materialize");
    // ctx.gfx.
    // let window = graphics::window(ctx);
    // let mut pos = window.get_position().unwrap();
    // pos.x = 0.0;
    // pos.y = 0.0;
    // Setup game state -> game loop
    let cf = Config {
        grid_width: 3000,
        grid_height: 2000,
        cell_size: 1_f32,
        screen_size: (1440_f32, 1080_f32),
        fps: 60_u32,
        // initial_state: "",
        terminal_display: false,
    };

    println!("Main device : {}", device.name().unwrap());
    // let mut planner = FftPlanner::new();
    // let fft = planner.plan_fft_forward(12000);
    // // fft.
    // let mut buffer = vec![];
    // println!("{:?}", b[0]);
    // for point in b {
    //     buffer.push(Complex32::new(point.x, point.y));
    // }
    // println!("{}", buffer[0]);
    // fft.process(&mut buffer);
    // println!("{}", buffer[0]);

    // let fft_values = to_points(&buffer);
    // let fc = fft_values.clone();
    // let (mut min_x, mut max_x) = (f32::MAX, f32::MIN);
    // let (mut min_y, mut max_y) = (f32::MAX, f32::MIN);
    // for val in fc {
    //     // 1 point uncertainty (if min === max, we get default value; but it avoids a bunch of
    //     //   overhead)
    //     if val.x < min_x {
    //         min_x = val.x
    //     }
    //     if val.x > max_x {
    //         max_x = val.x
    //     }
    //     if val.y < min_y {
    //         min_y = val.y
    //     }
    //     if val.y > max_y {
    //         max_y = val.y
    //     }
    // }
    // println!("sample : {}", sample_rate.0);
    // let mut ic: Vec<_> = c.iter().map(|x| return x.y).collect();
    // _ = ic.split_off(16384);
    // println!("{}", ic.len());
    // let res = samples_fft_to_spectrum(
    //     &ic,
    //     sample_rate.0,
    //     FrequencyLimit::All,
    //     // Recommended scaling/normalization by `rustfft`.
    //     Some(&divide_by_N_sqrt),
    // )
    // .unwrap();
    // let mapval = res.to_map();
    // let l = mapval.len();
    // let mut v: Vec<Vec<f32>> = vec![vec![], vec![], vec![], vec![], vec![]];
    // for (&key, value) in mapval.iter() {
    //     if key < (l / 5) as u32 {
    //         v[0].push(*value)
    //     } else if ((l / 5_usize) as u32) < key && key < ((2 * l / 5_usize) as u32) {
    //         v[1].push(*value)
    //     } else if ((2 * l / 5_usize) as u32) < key && key < ((3 * l / 5_usize) as u32) {
    //         v[2].push(*value)
    //     } else if ((3 * l / 5_usize) as u32) < key && key < ((4 * l / 5_usize) as u32) {
    //         v[3].push(*value)
    //     } else {
    //         v[4].push(*value)
    //     }
    //     //println!("key : {} - value : {}", key, value);
    // }
    // let mut useless_counter = 0;
    // for vector in v {
    //     useless_counter += 1;
    //     // let mut sum: f32 = 0.0;

    //     // for point in vector {
    //     //     sum += point
    //     // }
    //     let iter = vector.iter();
    //     let sum = iter.fold(0_f32, |x, &x2| x + x2);
    //     let mean = sum / vector.len() as f32;
    //     println!(
    //         "vector nr. {} : This is what we get : ({}) - [{}]",
    //         useless_counter, sum, mean
    //     )
    // }

    // println!("min-max : {}, {} ||| {}, {} ", min_x, max_x, min_y, max_y);
    let state = MainState::new(
        &mut ctx,
        sample_rate,
        cf,
        vec![vec![0.0], vec![0.0], vec![0.0], vec![0.0]],
        Arc::clone(&reader),
    ); //fft_values);
    event::run(ctx, event_loop, state);
    // drop(stream);
    // Ok(())
}

fn to_points(buffer: &Vec<Complex32>) -> Vec<Point2<f32>> {
    let mut hold = vec![];
    for value in buffer.iter() {
        hold.push(Point2 {
            x: value.re / 1_000_000_0.0,
            y: value.im / 100000.0,
        })
    }
    hold.sort_by(|p, n| return n.x.total_cmp(&p.x));
    return hold;
}

fn write_input_data<T, U>(input: &[T], reader: &Arc<Mutex<DataCollected<U>>>)
where
    DataCollected<U>: Reader<T, U>,
    T: Sample,
    U: Sample + cpal::FromSample<T> + std::fmt::Debug,
    f32: From<U>,
{
    // if let Ok(mut guard) = writer.try_lock() {
    let r = Arc::clone(&reader);
    let mut guard = r.lock().unwrap();
    guard.results_count += 1;
    guard.add(input);
    // for (i, &sample) in input.iter().enumerate() {
    //     // let reader_instance: DataCollected<T> = DataCollected::from_mutex::<T>(guard);
    //     let sample: U = U::from_sample(sample);
    //     guard.sample_count += 1;
    //     guard.set_min_max(sample);
    // }
    // // guard.spectrum.p
    // guard.results_total += input.len() as f32;
    // }
    // }
}

// TODO type archmutex data ?
struct MainState<T> {
    // grid: Grid,
    values: Vec<Vec<f32>>,
    // calculated: [f32; 5],
    reader: Arc<Mutex<DataCollected<T>>>,
    sample_rate: SampleRate,
    changed: bool,
    counter: [(f32, f32); 5],
    pos_x: f32,
}
// impl Into<mint::Point2<f32>>
impl MainState<u8> {
    fn new(
        _ctx: &mut Context,
        sample_rate: SampleRate,
        config: Config,
        values: Vec<Vec<f32>>,
        reader: Arc<Mutex<DataCollected<u8>>>,
    ) -> Self {
        // let grid = Grid::new(config.grid_width, config.grid_height);

        MainState {
            // grid,
            // config,
            changed: true,
            // calculated: [0.0, 0.0, 0.0, 0.0, 0.0],
            sample_rate,
            reader,
            values,
            counter: [(0.0, 0.0); 5],
            pos_x: 0_f32,
        }
    }

    fn sample(&mut self) {
        // println!("sampling, sampling, SAMPLING");
        let mut guard = self.reader.lock().unwrap();
        if guard.points.len() < 16384 {
            return;
        }
        let pts = guard.points.clone();
        let (sl, remain) = pts.split_at(16384);
        guard.points = remain.to_vec();
        let res = samples_fft_to_spectrum(
            &sl,
            self.sample_rate.0,
            FrequencyLimit::All,
            // Recommended scaling/normalization by `rustfft`.
            Some(&divide_by_N_sqrt),
        )
        .unwrap();
        let mapval = res.to_map();
        let l = mapval.len();
        let mut v: Vec<Vec<f32>> = vec![vec![], vec![], vec![], vec![], vec![]];
        for (&key, value) in mapval.iter() {
            if key < (l / 5) as u32 {
                v[0].push(*value)
            } else if ((l / 5_usize) as u32) < key && key < ((2 * l / 5_usize) as u32) {
                v[1].push(*value)
            } else if ((2 * l / 5_usize) as u32) < key && key < ((3 * l / 5_usize) as u32) {
                v[2].push(*value)
            } else if ((3 * l / 5_usize) as u32) < key && key < ((4 * l / 5_usize) as u32) {
                v[3].push(*value)
            } else {
                v[4].push(*value)
            }
        }
        if guard.points.len() < 16384 {
            guard.points = vec![];
        }
        self.values = v;
        self.changed = true;
        // println!("{:?}", self.values);
    }
}
impl event::EventHandler<ggez::GameError> for MainState<u8> {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.pos_x = self.pos_x + 2.0;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.0, 0.0, 0.0, 1.0]));
        let mode = DrawMode::Fill(FillOptions::DEFAULT);
        // let mode = DrawMode::Stroke(StrokeOptions::DEFAULT);
        // let t = graphics::Text::new("Hello world");
        // canvas.draw(&t, Vec2::new(0.0, 800.0));
        // let points = Vec2::new(0.0, 0.0);
        // println!("{} is the length : ", self.values.len());
        let mut x_position = 50_f32;
        const Y_POSITION: f32 = 200_f32;
        self.sample();
        let mut i = 0;
        for frequency in &self.values {
            let iter = frequency.iter();
            let sum = iter.fold(0_f32, |x, &x2| x + x2);
            self.counter[i].0 = (sum / frequency.len() as f32) * 4.0;
            // if self.counter[i].1 > self.counter[i].0 {
            //     self.counter[i].1 -= (self.counter[i].0 - self.counter[i].1) * 100.0 * (1.0 / 125.0)
            //     // self.counter[i].1 - 4.0
            // } else {
            self.counter[i].1 += (self.counter[i].0 - self.counter[i].1) * 20.0 * (1.0 / 125.0);

            // TODO
            // remove magic numbers (Speed & dt)

            let rectangle = graphics::Mesh::new_rounded_rectangle(
                ctx,
                mode,
                graphics::Rect::new(x_position, Y_POSITION, 100_f32, self.counter[i].1),
                20_f32,
                match i {
                    0 => graphics::Color::WHITE,
                    1 => graphics::Color::CYAN,
                    2 => graphics::Color::RED,
                    _ => graphics::Color::YELLOW,
                },
            )
            .unwrap();
            canvas.draw(&rectangle, Vec2::new(0_f32, -self.counter[i].1));

            i += 1;
            x_position += 160_f32;
        }
        // let line = graphics::Mesh::new_polyline(ctx, mode, &self.values, Color::WHITE)?;
        // let circle = graphics::Mesh::new_circle(
        //     ctx,
        //     graphics::DrawMode::fill(),
        //     Vec2::new(0.0, 0.0),
        //     100.0,
        //     2.0,
        //     Color::WHITE,
        // )?;
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

        canvas.finish(ctx)?;
        Ok(())
    }
}
struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}
impl Grid {
    // Width and height of the Grid
    pub fn new(width: usize, height: usize) -> Self {
        return Self {
            width,
            height,
            cells: vec![Cell::new(false); width * height],
        };
    }
}

#[derive(Clone, Debug)]
pub struct Cell {
    alive: bool,
}

impl Cell {
    pub fn new(alive: bool) -> Self {
        return Self { alive };
    }
}

#[derive(Debug, Clone)]
struct DataCollected<U> {
    display: Display,
    min: U,
    max: U,
    sample_count: u64,
    results_count: u64,
    results_total: f32,
    points: Vec<f32>,
}
// impl DataCollected<u8> {
//     fn set_min_max(&mut self, sample: u8) -> DataCollected<u8> {
//         if self.min > sample {
//             return DataCollected {
//                 display: self.copy_display(),
//                 min: sample,
//                 max: self.max,
//             };

//             // DataCollected {
//             //     min: sample,
//             //     max: self.min,
//             // }
//         } else if self.max < sample {
//             return DataCollected {
//                 display: self.copy_display(),
//                 min: self.min,
//                 max: sample,
//             };
//         } else {
//             return self.clone();
//         };
//     }

//     fn from_mutex(reader: MutexGuard<'_, &mut DataCollected<u8>>) -> DataCollected<u8> {
//         let (min, max) = (reader.min, reader.max);
//         return DataCollected {
//             display: reader.copy_display(),
//             min,
//             max,
//         };
//     }
//     fn copy_display(&self) -> Display {
//         let name: String = self.display.name.clone();
//         let size: Size = self.display.size.clone();
//         return Display::new();
//     }
// }
// impl DataCollected {
//     fn read_sample<T, U>(&self, sample: U) -> _ where T: Sample, U: Sample + cpal::FromSample<T> + std::fmt::Debug {
//         todo!()
//     }
// }
//
trait Reader<X, A> {
    fn read_sample<T, U>(&self, sample: U)
    where
        T: Sample,
        U: Sample + cpal::FromSample<T> + std::fmt::Debug;
    fn set_min_max(&mut self, sample: A) -> ();
    fn add(&mut self, samples: &[X]);
}
// impl Reader for T {
//     fn read_sample<T, U>(&self, sample: U)
//     where
//         T: Sample,
//         U: Sample + cpal::FromSample<T> + std::fmt::Debug,
//     {
//         println!("{:?}", sample)
//     }
// }
impl Reader<u8, u8> for DataCollected<u8> {
    fn read_sample<T, U>(&self, sample: U)
    where
        T: Sample,
        U: Sample + cpal::FromSample<T> + std::fmt::Debug,
    {
        print!("a");
    }

    fn set_min_max(&mut self, sample: u8) {
        if self.min > sample {
            self.min = sample;
        } else if self.max < sample {
            self.max = sample;
        }
    }
    fn add(&mut self, samples: &[u8]) {
        self.points.extend(samples.iter().map(|y| {
            return *y as f32 - 128.0; // * 10 ?
        }));
    }
}

// trait toDisplay {}

#[derive(Debug, Clone)]
struct Size {
    height: u8,
    width: u8,
}

#[derive(Debug, Clone)]
struct Display {
    name: String,
    size: Size,
}
impl Display {
    fn new() -> Display {
        return Display {
            name: String::from("console"),

            size: Size {
                height: 40,
                width: 50,
            },
        };
    }
}

// struct Point<T> {
//     x: T,
//     y: T,
// }

// impl Point<u8> {
//     fn new(x: u8, y: u8) -> Point<u8> {
//         return Point { x, y };
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
    // pub initial_state: String,
    pub terminal_display: bool,
}
