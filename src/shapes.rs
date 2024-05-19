// use core::panic;
use ggez::{
    glam::Vec2,
    graphics::{self, Canvas, Color},
};

#[derive(Debug)]
pub struct ShapeBuilder {}

#[derive(Clone)]
pub enum Shape {
    RoundedRectangular(RoundedRectangularShape),
    Cyclic(Cyclic),
}

impl ShapeBuilder {
    pub fn new(kind: ShapeKind) -> Shape {
        match kind {
            ShapeKind::RoundedRectangular => Shape::RoundedRectangular(RoundedRectangularShape {}),
            ShapeKind::Cyclic => Shape::Cyclic(Cyclic {}),
            // _ => panic!("What the hell ?"),
        }
    }
}

pub fn spiraling(len: usize) -> Vec<(f32, f32)> {
    let mut angle = 0.0_f32;
    let mut result: Vec<(f32, f32)> = vec![];
    for i_usize in 0..len {
        let i = i_usize as f32;
        let (s, c) = (angle.sin(), angle.cos());
        // println!("{} {}", s, c);
        let x = (i * 20.0) * c;
        let y = (i * 20.0) * s;
        result.push((x, y));
        // unimplemented!();

        angle += 0.9; // 0.4
    }
    result
}

#[derive(Debug)]
pub enum ShapeKind {
    RoundedRectangular,
    Cyclic,
}

#[derive(Clone)]
pub struct Cyclic {}
impl Cyclic {
    pub fn draw(
        &self,
        ctx: &mut ggez::Context,
        // i: usize,
        mode: graphics::DrawMode,
        mut color: Color,
        position: (f32, f32),
        size: (f32, f32),
        canvas: &mut Canvas,
    ) {
        let shape = graphics::Mesh::new_circle(
            ctx,
            mode,
            Vec2::new(position.0, position.1),
            50.0 + size.0,
            0.3,
            color,
        )
        .unwrap();
        color.a = 0.1;
        let after_image = graphics::Mesh::new_circle(
            ctx,
            mode,
            Vec2::new(position.0, position.1),
            (50.0 + size.1) * 2.0,
            0.3,
            color,
        )
        .unwrap();
        canvas.draw(&shape, Vec2::new(0_f32, 0_f32));
        canvas.draw(&after_image, Vec2::new(0_f32, 0_f32));
    }
}

#[derive(Clone)]
pub struct RoundedRectangularShape {}
impl RoundedRectangularShape {
    pub fn draw(
        &self,
        ctx: &mut ggez::Context,
        // i: usize,
        mode: graphics::DrawMode,
        mut color: Color,
        (x_position, y_position): (f32, f32),
        (size, afterimage_size): (f32, f32),
        radius: f32,

        canvas: &mut Canvas,
    ) {
        let shape = graphics::Mesh::new_rounded_rectangle(
            ctx,
            mode,
            graphics::Rect::new(x_position, y_position, 10_f32, size),
            radius,
            color,
        )
        .unwrap();
        color.a = 0.1;
        let after_image = graphics::Mesh::new_rounded_rectangle(
            ctx,
            mode,
            graphics::Rect::new(x_position, y_position, 10.50_f32, afterimage_size),
            radius,
            color,
        )
        .unwrap();
        canvas.draw(&shape, Vec2::new(0_f32, -size));
        canvas.draw(&after_image, Vec2::new(0_f32, -afterimage_size)); //TODO this needs to
    }
}
