use rust_on_rails::prelude::*;
use pelican_ui::prelude::*;
use pelican_ui::prelude::Text as Text;

use quircs::Quirc;
use image::{RgbaImage, DynamicImage};

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
        // let img_gray = image::ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        //     let Luma([l]) = img.get_pixel(x, y);
        //     if *l > 128 { Luma([255u8]) } else { Luma([0u8]) }
        // });

        let img_gray = DynamicImage::ImageRgba8(img).into_luma8();
        let codes = self.4.identify(img_gray.width() as usize, img_gray.height() as usize, &img_gray);

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
}

impl OnEvent for QRCodeScanner {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref() {
            let frame = self.3.get_frame();
            match frame {
                Ok(f) => {
                    if let Some(data) = self.find_code(f.clone()) {
                        println!("FOUND DATA, TRIGGERING EVENT");
                        ctx.trigger_event(QRCodeScannedEvent(data))
                    }
                    
                    *self.2.message() = None; *self.2.background() = None;
                    let image = ctx.add_image(f);
                    self.1 = Some(Image{shape: ShapeType::Rectangle(0.0, (300.0, 300.0)), image, color: None});
                },
                Err(CameraViewError::AccessDenied) => {
                    let background = ctx.get::<PelicanUI>().theme.colors.background.secondary;
                    *self.2.background() = Some(RoundedRectangle::new(0.0, 8.0, background));
                    *self.2.message() = Some(Message::new(ctx, "settings", "Enable camera in settings."));
                },
                Err(CameraViewError::FailedToGetFrame) => {
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