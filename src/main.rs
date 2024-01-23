// ffmpeg -framerate 30 -start_number 1 -i ./animation/%00d.png -pix_fmt yuv420p 30fps.mp4
// ffmpeg -framerate 60 -start_number 1 -i ./animation/%00d.png -pix_fmt yuv420p -crf 20 -c:v libx265 60fps2.mp4
// https://ottverse.com/cbr-crf-changing-resolution-using-ffmpeg/

#![allow(dead_code)]
// #![allow(unused_variables)]
// #![allow(unused_imports)]
// #![allow(unused_mut)]

use std::ops::Add;
use std::ops::Mul;
// use std::str::FromStr;
// use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::path::Path;

use bigdecimal::num_traits::MulAdd;
// use colorgrad::Color;
use colorgrad::Gradient;
use image::Rgb;
use image::RgbImage;
use rayon::prelude::*;
// use bigdecimal::BigDecimal;
use num_complex::Complex;
use indicatif::ProgressBar;

struct Mandelbrot{
    center: Complex<f64>,
    height: u32,
    width: u32,
    // total_frames: u16,
    color_pallette: Gradient,
    base_size: f64,
    aa_samples: u8,
    write_images: bool,
    overwrite: bool
}


impl Mandelbrot{
    fn run_singlethreaded(&self, zoom_level: f64, filename: String)-> u64{

        let string_path = "./animation/".to_string() + &filename + ".png";


        if !self.overwrite{
            if Path::new(&string_path).is_file(){
                return 0;
            }
        }

        let mut img = RgbImage::new(self.width, self.height);
        let aspect_ratio: f64 = self.height as f64 / self.width as f64;
        let size_x = f64::powf(2.0, -1.0 * zoom_level) * self.base_size;
        let size_y = f64::powf(2.0, -1.0 * zoom_level) * (self.base_size * aspect_ratio);

        let min_real: f64 = self.center.re - (size_x / 2.0);
        // let max_real: f64 = self.center.re + (size_x / 2.0);

        // let min_img: f64 = self.center.im - (size_y / 2.0);
        let max_img: f64 = self.center.im + (size_y / 2.0);


        let top_left     = Complex::new(min_real, max_img);
        // let bottom_right = Complex::new(max_real, min_img);

        let max_iterations = calculate_max_iterations(self.width as f64 / size_x) as i32;

        let mut iterations_preformed: u64 = 0;

        for y in 0..self.height{
            for x in 0..self.width{
                let point: Vec<Complex<f64>> = self.pixel_to_point_aa(x, y, size_x, size_y, &top_left, self.aa_samples);
                let pixel_value: i32 = self.iterations_aa(&point, max_iterations);
                iterations_preformed = iterations_preformed + (pixel_value * (point.len()) as i32) as u64;
                let pixel = get_pixel_color(&self.color_pallette, max_iterations, pixel_value);
                img.put_pixel(x as u32, y as u32, pixel);
            }
        }

        if self.write_images{
            img.save(string_path).unwrap();
        }

        return iterations_preformed;

    }


    fn pixel_to_point(&self, x: u32, y: u32, size_x: f64, size_y: f64, top_left: &Complex<f64>) -> Complex<f64>{
        let pixel_width  = size_x / self.width as f64;
        let pixel_height = size_y / self.height as f64;

        return Complex::new(top_left.re + (pixel_width * x as f64), top_left.im - (pixel_height * y as f64) );

    }

    fn pixel_to_point_aa(&self, x: u32, y: u32, size_x: f64, size_y: f64, top_left: &Complex<f64>, aa_samples: u8) -> Vec<Complex<f64>>{

        let mut return_vec = Vec::new();
        let pixel_width  = size_x / self.width as f64;
        let pixel_height = size_y / self.height as f64;

        let sub_pixel_width  = pixel_width / aa_samples as f64;
        let sub_pixel_height = pixel_height / aa_samples as f64;

        for sub_y in 0..aa_samples{
            for sub_x in 0..aa_samples{
                return_vec.push(
                    Complex::new ( 
                        top_left.re + (pixel_width * x as f64) + sub_pixel_width * sub_x as f64, 
                        top_left.im - (pixel_height * y as f64)+ sub_pixel_height * sub_y as f64)
                    );
            }
        }

        return return_vec;

    }

    fn get_pixel_color_raw(&self, max_iteration: i32, iter_count: i32) -> Vec<u8>{

        let interpolation_value = iter_count as f64 / max_iteration as f64;
        let color = self.color_pallette.at(interpolation_value).to_rgba8();
    
        return vec![color[0], color[1], color[2]];
    }

    fn iterations_aa(&self, points: &Vec<Complex<f64>>, max_iteration: i32) -> i32{
        return (points.iter().map(|f: &Complex<f64>| self.iterations(f, max_iteration) as f64).sum::<f64>() / points.len() as f64) as i32;
    }

    fn iterations(&self, point: &Complex<f64>, max_iteration: i32) -> i32{
        let mut z = Complex::new(0.0, 0.0);

        // let mut iteration = 0;
        // while z.norm_sqr() < 4.0 && iteration < max_iteration{
        //     z = z.mul(z);
        //     z = z.add(point);
        //     iteration = iteration + 1;
        // }

        for i in 0..max_iteration {
            // z = z.mul(z);
            // z = z.add(point);

            z = z.mul_add(z, *point);

            if z.norm_sqr() > 4.0{
                return i
            }
        }

        return max_iteration;
    }

    // fn iterations(&self, point: &Complex<f64>, max_iteration: i32) -> i32{
    //     let mut z_re:  f64 = 0.0;
    //     let mut z_img: f64 = 0.0;

    //     let mut z_re_sqr: f64 = 0.0;
    //     let mut z_im_sqr: f64 = 0.0;

    //     for i in 0..max_iteration {
    //         z_re_sqr = z_re*z_re;
    //         z_im_sqr = z_img * z_img;

    //         z_re = (z_re_sqr   + (z_im_sqr * -1.0)) + point.re;
    //         z_img = (z_im_sqr * z_re * 2.0) + point.im;

    //         if ((z_img * z_img) + (z_re * z_re)) > 16.0{
    //             return i
    //         }

    //     }


    //     return max_iteration;
            
    // }


}
fn get_pixel_color(gradient: &Gradient, max_iteration: i32, iter_count: i32) -> Rgb<u8>{

    let interpolation_value = iter_count as f64 / max_iteration as f64;
    let color = gradient.at(interpolation_value).to_rgba8();

    return Rgb([color[0], color[1], color[2]]);
}

fn calculate_max_iterations(zoom_level: f64) -> f64{
    return 50.0 * (zoom_level.log10()).powf(1.5);
}

fn separate(input: u64) -> String{
    return input.to_string()
                            .as_bytes()
                            .rchunks(3)
                            .rev()
                            .map(std::str::from_utf8)
                            .collect::<Result<Vec<&str>, _>>()
                            .unwrap()
                            .join(",");  // separator
}



fn run(){
    let frame = Mandelbrot {
        width: 1920,
        height: 1080,
        center: Complex::new(-0.811723852, -0.169001979 ),
        color_pallette: colorgrad::rainbow(),
        // total_frames: 32,
        base_size: 3.0,
        aa_samples: 2,
        write_images: false,
        overwrite: true

    };

    // Re: -0.811723852
    // Im: -0.169001979
    let starting_zoom = 0.0;
    let ending_zoom  = 32.0;
    let frames = 16;
    let step = (ending_zoom - starting_zoom) / frames as f64;

    let queue_start = Instant::now();

    let bar = ProgressBar::new(frames);


    (0..frames).into_par_iter().for_each(|i| {
        let zoom_level = i as f64 * step;
        let start = Instant::now();
        let iterations: u64 = frame.run_singlethreaded(zoom_level, i.to_string());
        let duration = start.elapsed();

        // let mut num = counter.lock().unwrap();
        bar.inc(1);
        // *num += 1;
        // println!("{}/{} frames done in: {:?}! Speed: {} iterations/second", bar.position(), frames, duration, iterations as f64 / duration.as_secs_f64());

        println!("{}/{} frames done in: {:?}!\nCalculations: {} iterations preformed\nSpeed: {} iterations/second", bar.position(), frames, duration, separate(iterations), separate((iterations as f64 / duration.as_secs_f64()) as u64));
    });

    bar.finish();
    let total_duration = queue_start.elapsed();
    println!("Completed render in: {:?}", total_duration);
}

fn main() {
    run();
}
