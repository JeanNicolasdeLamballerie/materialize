// struct StreamState {
//     current: Vec<Point2<f32>>,
//     next: Vec<Point2<f32>>,
//     previous: Vec<Point2<f32>>,
//     config: Config,
//     pos_x: f32,
// }
// impl StreamState {
//     fn new(_ctx: &mut Context, config: Config, current: Vec<Point2<f32>>) -> Self {
//         StreamState {
//             config,
//             next: current,
//             previous: current,
//             current,
//             pos_x: 0_f32,
//             // pos_y: 0_f32,
//         }
//     }
// }
// trait Reader<X, A> {
//     fn read_sample<T, U>(&self, sample: U)
//     where
//         T: Sample,
//         U: Sample + cpal::FromSample<T> + std::fmt::Debug;
//     fn set_min_max(&mut self, sample: A) -> ();
//     fn add(&mut self, samples: &[X]);
// }

// impl Reader<u8, u8> for DataCollected<u8> {
//     fn read_sample<T, U>(&self, sample: U)
//     where
//         T: Sample,
//         U: Sample + cpal::FromSample<T> + std::fmt::Debug,
//     {
//         print!("a");
//     }

//     fn set_min_max(&mut self, sample: u8) {
//         if self.min > sample {
//             self.min = sample;
//         } else if self.max < sample {
//             self.max = sample;
//         }
//     }
//     fn add(&mut self, samples: &[u8]) {
//         self.points.extend(samples.iter().enumerate().map(|(x, y)| {
//             return Point2 {
//                 x: self.results_total + x as f32,
//                 y: (*y as f32 - 128.0) * 10.0,
//             };
//         }));
//     }
// }
