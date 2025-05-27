use rust_on_rails::prelude::*;
use pelican_ui::prelude::*;
use pelican_ui::prelude::Text as Text;
use image::{DynamicImage, GrayImage, Luma, Rgba, RgbaImage};
use imageproc::region_labelling::{connected_components, Connectivity};
use imageproc::contours::find_contours_with_threshold;
use image::imageops::invert;
use imageproc::contrast::ThresholdType;
use imageproc::contrast::threshold;
use imageproc::contours::BorderType;
use imageproc::contours::Contour;
use imageproc::drawing::draw_filled_rect_mut;
use imageproc::rect::Rect;
use std::path::Path;

use opencv::{
    core::{Point, Point2f, Scalar, Vector, AlgorithmHint, BORDER_DEFAULT, BORDER_CONSTANT},
    imgcodecs,
    imgproc,
    prelude::*,
    Result,
};

use quircs::Quirc;

use crate::events::QRCodeScannedEvent;

/// A component for scanning QR codes using the device camera.
#[derive(Debug, Component)]
pub struct QRCodeScanner(Stack, Option<Image>, QRGuide, #[skip] Camera, #[skip] Quirc);

impl QRCodeScanner {
    /// Creates a new `QRCodeScanner` component with a centered stack layout, a QR guide, and a camera instance.
    ///
    /// # Parameters
    /// - `ctx`: The [`Context`] for accessing the app's theme.
    ///
    /// # Example
    /// ```
    /// let scanner = QRCodeScanner::new(ctx);
    /// ```
    pub fn new(ctx: &mut Context) -> Self {
        QRCodeScanner(Stack::center(), None, QRGuide::new(ctx), Camera::new(), Quirc::default())
    }

    fn find_code(&mut self, img: RgbaImage) -> Option<String> {
        let img = image::open(&Path::new("assets/test.png")).expect("Failed to open input image");
        // let img = DynamicImage::ImageRgba8(img);
        let output = make_readable(img);
        decode_image(output.to_luma8(), self.4.clone())
    }
}

impl OnEvent for QRCodeScanner {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref() {
            let frame = self.3.get_frame();
            match frame {
                Ok(f) => {
                    
                    // if let Some(data) = self.find_code(f.clone()) {
                    //     println!("FOUND DATA, TRIGGERING EVENT");
                    //     ctx.trigger_event(QRCodeScannedEvent(data))
                    // }

                    // find_circles();
                    
                    *self.2.message() = None; *self.2.background() = None;
                    let image = ctx.add_image(f);
                    self.1 = Some(Image{shape: ShapeType::Rectangle(0.0, (300.0, 300.0)), image, color: None});
                },
                Err(CameraError::AccessDenied) => {
                    let background = ctx.get::<PelicanUI>().theme.colors.background.secondary;
                    *self.2.background() = Some(RoundedRectangle::new(0.0, 8.0, background));
                    *self.2.message() = Some(Message::new(ctx, "settings", "Enable camera in settings."));
                },
                Err(CameraError::FailedToGetFrame) | Err(CameraError::WaitingForAccess) => {
                    let background = ctx.get::<PelicanUI>().theme.colors.background.secondary;
                    *self.2.background() = Some(RoundedRectangle::new(0.0, 8.0, background));
                    *self.2.message() = Some(Message::new(ctx, "camera", "Accessing device camera."));
                }
            }
        }
        true
    }
}


#[derive(Debug, Component)]
struct QRGuide(Stack, Option<RoundedRectangle>, RoundedRectangle, Option<Message>);
impl OnEvent for QRGuide {}

impl QRGuide {
    pub fn new(ctx: &mut Context) -> Self {
        let colors = ctx.get::<PelicanUI>().theme.colors;
        let (background, color) = (colors.background.secondary, colors.outline.secondary);
        QRGuide(
            Stack(Offset::Center, Offset::Center, Size::Static(308.0), Size::Static(308.0), Padding::default()), 
            Some(RoundedRectangle::new(0.0, 8.0, background)), 
            RoundedRectangle::new(4.0, 8.0, color), 
            Some(Message::new(ctx, "camera", "Accessing device camera."))
        )
    }

    pub fn message(&mut self) -> &mut Option<Message> {&mut self.3}
    pub fn background(&mut self) -> &mut Option<RoundedRectangle> {&mut self.1}
}

#[derive(Debug, Component)]
struct Message(Column, Image, Text);
impl OnEvent for Message {}

impl Message {
    pub fn new(ctx: &mut Context, icon: &'static str, msg: &str) -> Self {
        let theme = &ctx.get::<PelicanUI>().theme;
        let (color, font_size) = (theme.colors.shades.lighten, theme.fonts.size.sm);
        Message(Column::center(4.0), 
            Icon::new(ctx, icon, color, 48.0),
            Text::new(ctx, msg, TextStyle::Secondary, font_size, Align::Left)
        )
    }
}

fn decode_image(img_gray: GrayImage, mut decoder: Quirc) -> Option<String> {
    let codes = decoder.identify(img_gray.width() as usize, img_gray.height() as usize, &img_gray);

    for code in codes {
        match code {
            Ok(c) => {
                match c.decode() {
                    Ok(decoded) => {
                        let code = std::str::from_utf8(&decoded.payload).unwrap();
                        println!("qrcode: {}", code);
                        return Some(code.to_string());
                    }
                    Err(e) => println!("COULD NOT DECODE {:?}", e)
                }
            }
            Err(e) => println!("COULD NOT UNWRAP {:?}", e)
        }
    }
    None
}

fn make_readable(img: DynamicImage) -> DynamicImage {
    let bw = to_black_and_white(&img, 200);
    bw.save("bw.png");
    let cro = crop(&bw).expect("yeah, no");
    cro.save("final_crop.png");

    let labels = connected_components(&cro, Connectivity::Four, Luma([0]));
    let max_label = labels.pixels().map(|p| p.0[0]).max().unwrap_or(0);
    println!("Detected {} modules/blobs", max_label);

    let mut centroids = vec![(0u32, 0u32, 0u32); max_label as usize + 1];

    for (x, y, label) in labels.enumerate_pixels() {
        let l = label.0[0] as usize;
        if l == 0 {
            continue;
        }
        let (sum_x, sum_y, count) = centroids[l];
        centroids[l] = (sum_x + x, sum_y + y, count + 1);
    }

    let mut centroids_pos: Vec<(f32, f32)> = centroids.iter()
        .skip(1)
        .map(|&(sum_x, sum_y, count)| {
            (sum_x as f32 / count as f32, sum_y as f32 / count as f32)
        })
        .collect();

    centroids_pos.remove(0);

    // let mut save = DynamicImage::ImageRgba8(cro.to_rgba8());
    // let mut white_bg_img = RgbaImage::new(bw.width(), bw.height());
    // for pixel in white_bg_img.pixels_mut() {
    //     *pixel = Rgba([255, 255, 255, 255]);
    // }

    let rgba_buf = DynamicImage::ImageLuma8(cro).to_rgba8();
    let mut save = DynamicImage::ImageRgba8(rgba_buf);

    let square_size: u32 = 20;
    let half_size = (square_size / 2) as i32;

    for &(cx, cy) in &centroids_pos {
        let rect = Rect::at(cx as i32 - half_size, cy as i32 - half_size)
            .of_size(square_size, square_size);
        draw_filled_rect_mut(&mut save, rect, Rgba([255, 0, 0, 255]));
    }

    // let save = DynamicImage::ImageRgba8(final_img);
    save.save("output.png").expect("Failed to save red dots on white image");
    println!("Saved red dots: codes/modules_marked.png");
    // DynamicImage::ImageRgba8(final_img)
    save
}

fn to_black_and_white(image: &DynamicImage, threshold: u8) -> GrayImage {
    let gray = image.to_luma8();
    let mut bw = GrayImage::new(gray.width(), gray.height());

    for (x, y, pixel) in gray.enumerate_pixels() {
        let luma = pixel[0];
        let value = if luma > threshold { 255 } else { 0 };
        bw.put_pixel(x, y, Luma([value]));
    }

    bw
}

fn invert_bw(image: &GrayImage) -> GrayImage {
    let mut inverted = image.clone();
    for pixel in inverted.pixels_mut() {
        pixel.0[0] = 255 - pixel.0[0];
    }
    inverted
}

fn crop(img: &GrayImage) -> Option<GrayImage> {
    let (width, height) = img.dimensions();

    // let mut binarized: GrayImage = threshold(&img, 128, ThresholdType::Binary);
    // binarized.save("binarized.png");

    // invert(&mut binarized);
    // binarized.save("inverted.png");

    let contours: Vec<Contour<u32>> = find_contours_with_threshold(&img, 100);

    let mut best_rect: Option<Rect> = None;
    let mut max_area = 0;

    for contour in contours {
        let points = &contour.points;

        // Manually compute bounding box
        let (min_x, min_y) = points.iter().fold((u32::MAX, u32::MAX), |(x, y), p| {
            (x.min(p.x), y.min(p.y))
        });
        let (max_x, max_y) = points.iter().fold((0, 0), |(x, y), p| {
            (x.max(p.x), y.max(p.y))
        });

        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;
        let area = width * height;
        let aspect_ratio = width as f32 / height as f32;

        if area > max_area && aspect_ratio > 0.7 && aspect_ratio < 1.3 {
            best_rect = Some(Rect::at(min_x as i32, min_y as i32).of_size(width, height));
            max_area = area;
        }
    }

    if let Some(rect) = best_rect {
        println!("QR code bounds: {:?}", rect);
        let cropped = image::imageops::crop_imm(img, rect.left() as u32, rect.top() as u32, rect.width() as u32, rect.height() as u32).to_image();
        cropped.save("qr_dynamic_crop.png").unwrap();
        return Some(cropped);
    } else {
        println!("No suitable QR code region found.");
        return None;
    }
}


fn find_circles() {
    let mut img = imgcodecs::imread("assets/qr_dynamic_crop.png", imgcodecs::IMREAD_COLOR).expect("get image");

    let mut gray = Mat::default();
    imgproc::cvt_color(&img, &mut gray, imgproc::COLOR_BGR2GRAY, 0, AlgorithmHint::ALGO_HINT_DEFAULT).expect("cvt color");

    let mut blurred = Mat::default();
    imgproc::gaussian_blur(&gray, &mut blurred, opencv::core::Size::new(5, 5), 1.5, 1.5, BORDER_DEFAULT, AlgorithmHint::ALGO_HINT_DEFAULT).expect("olo");
    imgcodecs::imwrite("blurred.png", &blurred, &Vector::new()).expect("blurred");

    let mut edges = Mat::default();
    imgproc::canny(&blurred, &mut edges, 30.0, 150.0, 3, false).expect("canny");
    imgcodecs::imwrite("edges.png", &edges, &Vector::new()).expect("could not save edges");

    let mut contours = Vector::<Vector<Point>>::new();
    imgproc::find_contours(
        &edges,
        &mut contours,
        imgproc::RETR_TREE,
        imgproc::CHAIN_APPROX_SIMPLE,
        Point::new(0, 0),
    ).expect("find contuors");

    let areas: Vec<f64> = contours.iter()
        .map(|c| imgproc::contour_area(&c, false).expect("area"))
        .filter(|&area| area > 100.0) 
        .collect();

    let sum: f64 = areas.iter().sum();
    let avg = sum / areas.len() as f64;

    println!("average = {:?}", avg);

    for contour in contours.iter() {
        let area = imgproc::contour_area(&contour, false).expect("area");
        if area < avg - 1000.0 || area > avg + 1000.0 {continue;}

        let mut approx = Vector::<Point>::new();
        imgproc::approx_poly_dp(&contour, &mut approx, 0.08 * imgproc::arc_length(&contour, true).expect("could not"), true).expect("approx poly dp");

        let mut center = Point2f::default();
        let mut radius = 0.0;
        imgproc::min_enclosing_circle(&contour, &mut center, &mut radius).expect("could not min enclosing");

        imgproc::circle(
            &mut img,
            Point::new(center.x as i32, center.y as i32),
            radius as i32,
            Scalar::new(0.0, 0.0, 255.0, 0.0),
            3,
            imgproc::LINE_8,
            0,
        ).expect("make circle");
        
    }

    imgcodecs::imwrite("output.png", &img, &Vector::new()).expect("could not write output");
}
