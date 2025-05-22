use rust_on_rails::prelude::*;
use pelican_ui::prelude::*;
use pelican_ui::prelude::Text as Text;


use qrcode::QrCode;

/// A component representing a QR code with a branded logo.
#[derive(Debug, Component)]
pub struct QRCode(Stack, Bin<Stack, RoundedRectangle>, Image);
impl OnEvent for QRCode {}

impl QRCode {
    /// Creates a new `QRCode` component with a QR code and a branded logo.
    ///
    /// # Parameters
    /// - `ctx`: The [`Context`] for accessing the app's theme.
    /// - `data`: The data to encode in the QR code.
    ///
    /// # Example
    /// ```
    /// let qr_code = QRCode::new(ctx, "https://example.com");
    /// ```
    pub fn new(ctx: &mut Context, data: &str) -> Self {
        let theme = &ctx.get::<PelicanUI>().theme;
        let (_app_icon, color) = (theme.brand.app_icon.clone(), theme.colors.shades.white);
        let qr_size = 275.0;
        // let logo_size = 72.0;

        let code = QrCode::new(data).unwrap(); // temp
        let gray_image = code.render::<image::Luma<u8>>().build(); // temp
        let (width, height) = gray_image.dimensions(); // temp
        let (width, height) = (width.saturating_sub(32), height.saturating_sub(32)); // temp
        let img = image::imageops::crop_imm(&gray_image, 16, 16, width, height).to_image(); // temp

        let rgba_image = image::ImageBuffer::from_fn(width, height, |x, y| {
            let image::Luma([luma]) = img.get_pixel(x, y);
            image::Rgba([*luma, *luma, *luma, 255])
        });

        QRCode (
            Stack::center(),
            Bin(
                Stack(Offset::Center, Offset::Center, Size::Static(qr_size), Size::Static(qr_size), Padding::default()),
                RoundedRectangle::new(0.0, 8.0, color),
            ),
            Image{shape: ShapeType::RoundedRectangle(0.0, (qr_size, qr_size), 8.0), image: ctx.add_image(rgba_image), color: None}
            // QRModules::new(ctx, data, qr_size, logo_size),  - NO CUSTOM STYLIZATION FOR THIS RELEASE
            // Brand::new(app_icon, (logo_size, logo_size)) - NO ICON FOR THIS RELEASE
        )
    }
}


// #[derive(Debug, Component)]
// struct QRModules(Column, Vec<QRModuleRow>);
// impl OnEvent for QRModules {}

// impl QRModules {
//     fn new(ctx: &mut Context, code_str: &str, qr_size: f32, logo_size: f32) -> Self {
//         let code = QrCode::new(code_str).unwrap();
//         let module_count = code.width() as u32;
//         let module_size = qr_size / module_count as f32;
    
//         let mut rows: Vec<QRModuleRow> = vec![];
//         for y in 0..module_count {
//             rows.push(QRModuleRow::new(
//                 ctx,
//                 code.clone(),
//                 module_count,
//                 logo_size,
//                 module_size,
//                 y,
//             ));
//         }
    
//         QRModules(Column::center(0.0), rows)
//     }
    
// }

// #[derive(Debug, Component)]
// struct QRModuleRow(Row, Vec<Shape>);
// impl OnEvent for QRModuleRow{}
// impl QRModuleRow {
//     fn new(
//         ctx: &mut Context, 
//         code: QrCode, 
//         module_count: u32, 
//         logo_size: f32,  
//         module_size: f32,
//         y: u32
//     ) -> Self {
//         let shades = &ctx.get::<PelicanUI>().theme.colors.shades;
    
//         let logo_modules = (logo_size / module_size) + 2.0;
//         let logo_start = (module_count - logo_modules as u32) / 2;
//         let logo_end = logo_start + logo_modules as u32;
    
//         let mut modules: Vec<Shape> = vec![];
    
//         for x in 0..module_count {
//             let color = 
//                 if x >= logo_start && x < logo_end &&
//                    y >= logo_start && y < logo_end
//                 {
//                     shades.transparent
//                 } else if code[(x as usize, y as usize)] == Color::Dark {
//                     shades.black
//                 } else {
//                     shades.transparent
//                 };
//             println!("circle size: {:?}", module_size);
//             let circle = Circle::new(module_size, color);
    
//             modules.push(circle);
//         }
//         QRModuleRow(Row::center(0.0), modules)
//     }
    
// }