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

use std::sync::{Mutex, Arc};
use std::sync::atomic::{AtomicBool, Ordering};


use quircs::Quirc;
// use std::sync::mpsc::{self, Receiver, Sender};

use crate::events::QRCodeScannedEvent;

/// A component for scanning QR codes using the device camera.
#[derive(Debug, Component)]
pub struct QRCodeScanner(Stack, Option<Image>, QRGuide, #[skip] Camera, #[skip] Arc<Mutex<Option<String>>>, #[skip] Arc<Mutex<bool>>);

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
        QRCodeScanner(Stack::center(), None, QRGuide::new(ctx), Camera::new(), Arc::new(Mutex::new(None)), Arc::new(Mutex::new(false)))
    }

    fn find_code(&mut self, img: RgbaImage) {
        if *self.5.lock().unwrap() {return;}
        *self.5.lock().unwrap() = true;

        let result_clone = self.4.clone();
        let flag_clone = self.5.clone();

        std::thread::spawn(move || {
            let img = DynamicImage::ImageRgba8(img);
            let result = decode_image(img.to_luma8(), Quirc::default());

            if let Some(r) = result {
                *result_clone.lock().unwrap() = Some(r);
            }

            *flag_clone.lock().unwrap() = false;
        });
    }

}

impl OnEvent for QRCodeScanner {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref() {
            let frame = self.3.get_frame();
            match frame {
                Ok(f) => {
                    
                    self.find_code(f.clone());
                    if let Some(data) = &*self.4.lock().unwrap() {
                        ctx.trigger_event(QRCodeScannedEvent(data.to_string()));
                    }
                    
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

//  if let Ok(bytes) = self.3.try_recv() {
//         let avatar = Avatar::new(
//             ctx,
//             AvatarContent::Icon("profile", AvatarIconStyle::Secondary),
//             Some(("edit", AvatarIconStyle::Secondary)),
//             false,
//             128.0,
//             Some(Box::new(move |ctx: &mut Context| {
//                 ctx.open_photo_picker(sender.clone());
//             })),
//         );

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
    img_gray.save("test.png");
    let codes = decoder.identify(img_gray.width() as usize, img_gray.height() as usize, &img_gray);

    for code in codes {
        // println!("Code {:?}", code);
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
    println!("ERROR OR NO CODE");
    None
}
