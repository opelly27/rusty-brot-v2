#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use colorgrad::Color;
use colorgrad::Gradient;
use image::Rgb;
use image::RgbImage;
use rayon::prelude::*;
use bigdecimal::BigDecimal;
use num_complex::Complex;

struct MandelbrotHP{
    center: Complex<BigDecimal>,
    height: u32,
    width: u32,
    frames: u32,
    color_pallette: Gradient,
    base_size: BigDecimal,
    aa_samples: i32,
    dry_run: bool
}

impl MandelbrotHP{
    fn run_singlethreaded(&self, zoom_level: BigDecimal, filename: String){
        let mut img = RgbImage::new(self.width, self.height);
        let aspect_ratio: f64 = self.height as f64 / self.width as f64;
        let size_x = TWO.powf(-1.0 * zoom_level) * self.base_size;
        let size_y = TWO.powf(-1.0 * zoom_level) * (self.base_size * aspect_ratio);

        let min_real: f64 = self.center.re - (size_x / TWO);
        let max_real: f64 = self.center.re + (size_x / TWO);

        let min_img: f64 = self.center.im - (size_y / TWO);
        let max_img: f64 = self.center.im + (size_y / TWO);


        let top_left     = Complex::new(min_real, max_img);
        let bottom_right = Complex::new(max_real, min_img);

        let mut max_iterations = calculate_max_iterations(self.width as f64 / size_x) as i32;

        for y in 0..self.height{
            for x in 0..self.width{
                let point: Vec<Complex<f64>> = self.pixel_to_point_aa(x, y, size_x, size_y, &top_left, self.aa_samples);
                let pixel_value: i32 = self.iterations_aa(point, max_iterations);
                let pixel = get_pixel_color(&self.color_pallette, max_iterations, pixel_value);
                img.put_pixel(x as u32, y as u32, pixel);
            }
        }

        if !self.dry_run{
            img.save("./animation/".to_string() + &filename + ".png").unwrap();
        }

    }
}