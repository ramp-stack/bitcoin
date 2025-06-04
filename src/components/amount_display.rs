use pelican_ui::events::{Event, OnEvent, Key, NamedKey, KeyboardState, KeyboardEvent};
use pelican_ui::drawable::{Drawable, Component, Align, Span, Image};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use pelican_ui_std::{
    TextStyle,
    Text,
    Row,
    IS_MOBILE,
    Padding,
    Size,
    Offset,
    Column,
    Stack,
    Icon
};

/// A component displaying an amount in both USD and BTC formats.
///
/// The [`AmountDisplay`] component is used to show financial information, such as
/// a currency amount in both USD and Bitcoin (BTC). It consists of a column layout
/// containing a main text for the USD amount and a subtext for the BTC equivalent.
#[derive(Debug, Component)]
pub struct AmountDisplay(Column, Text, SubText);
impl OnEvent for AmountDisplay {}

impl AmountDisplay {
    /// Creates a new [`AmountDisplay`] component.
    ///
    /// This function initializes the [`AmountDisplay`] with a column layout,
    /// a text component showing the USD amount, and a subtext component displaying
    /// the BTC equivalent.
    ///
    /// # Parameters
    /// - `ctx`: A mutable reference to [`Context`].
    ///
    /// # Returns
    /// A new [`AmountDisplay`] component with default values.
    ///
    /// # Example
    /// ```rust
    /// let amount_display = AmountDisplay::new(ctx);
    /// ```
    pub fn new(ctx: &mut Context, text: &str, subtext: &str) -> Self {
        let font_size = ctx.theme.fonts.size.title;

        AmountDisplay (
            Column::new(16.0, Offset::Center, Size::Fit, Padding(16.0, 64.0, 16.0, 64.0)),
            Text::new(ctx, text, TextStyle::Heading, font_size, Align::Left),
            SubText::new(ctx, subtext)
        )
    }

    /// Returns a mutable reference to the USD text in the [`AmountDisplay`].
    pub fn usd(&mut self) -> &mut String { &mut self.1.text().spans[0].text }

    /// Returns a mutable reference to the BTC text in the [`AmountDisplay`].
    pub fn btc(&mut self) -> &mut String { &mut self.2.2.text().spans[0].text }
}


#[derive(Debug, Component)]
struct SubText(Row, Option<Image>, Text, #[skip] bool);
impl OnEvent for SubText {}

impl SubText {
    fn new(ctx: &mut Context, btc: &str) -> Self {
        let text_size = ctx.theme.fonts.size.lg;
        SubText(Row::center(8.0), None, Text::new(ctx, btc, TextStyle::Secondary, text_size, Align::Left), true)
    }

    fn set_error(&mut self, ctx: &mut Context, err: &str) {
        let theme = &ctx.theme;
        let (color, text_size) = (theme.colors.status.danger, theme.fonts.size.lg);
        self.1 = Some(Icon::new(ctx, "error", color, 24.0));
        self.2 = Text::new(ctx, err, TextStyle::Error, text_size, Align::Left);
    }

    fn set_subtext(&mut self, ctx: &mut Context, txt: &str) {
        let text_size = ctx.theme.fonts.size.lg;
        self.1 = None;
        self.2 = Text::new(ctx, txt, TextStyle::Secondary, text_size, Align::Left);
    }

    fn error(&mut self) -> &mut bool {&mut self.3}
    fn _text(&mut self) -> &mut String {&mut self.2.text().spans[0].text}
}

/// A component for inputting an amount in USD and viewing BTC equivalent.
#[derive(Debug, Component)]
pub struct AmountInput(Stack, AmountInputContent);
impl OnEvent for AmountInput {}

impl AmountInput {
    /// Creates a new `AmountInput` component. Optionally takes in a preset usd amount.
    ///
    /// # Example
    /// ```
    /// let mut amount_input = AmountInput::new(ctx, None);
    /// ```
    pub fn new(ctx: &mut Context, usd: Option<(f64, &str)> ) -> Self {
        AmountInput (
            Stack(Offset::Center, Offset::Center, Size::Fit, Size::fill(), Padding::default()),
            AmountInputContent::new(ctx, usd),
        )
    }

    /// Returns the USD input value as a string.
    pub fn usd(&mut self) -> String { self.1.1.value() }
    /// Returns a mutable reference to the BTC input value.
    pub fn btc(&mut self) -> &mut f32 { &mut self.1.4 }
    /// Returns the bitcoin price set for calculating btc equivalent.
    pub fn price(&mut self) -> &mut f32 { &mut self.1.5 }
    /// Returns a mutable reference to the error flag.
    pub fn error(&mut self) -> &mut bool { self.1.2.error() }
    /// Sets the minimum value for the amount input.
    pub fn set_min(&mut self, a: f32) { self.1.3.0 = a; }
    /// Sets the maximum value for the amount input.
    pub fn set_max(&mut self, a: f32) { self.1.3.1 = a; }
    /// Validates the input field against checks.
    pub fn validate(&mut self, ctx: &mut Context) { self.1.validate(ctx) }
}

#[derive(Debug, Component)]
struct AmountInputContent(Column, Display, SubText, #[skip] (f32, f32), #[skip] f32, #[skip] f32);
// layout, display, subtext, (min, max fee), btc_price, btc input
impl AmountInputContent {
    fn new(ctx: &mut Context, usd: Option<(f64, &str)>) -> Self { // usd, nans
        let (num, sub) = usd.map(|(d, n)| {
            if d == 0.0 {
                ("0".to_string(), "Type dollar amount.".to_string())
            } else {
                let s = format!("{:.2}", d);
                match &s[s.len() - 2..] == "00" {
                    true => (s[..1].to_string(), n.to_string()),
                    false => (s.to_string(), n.to_string()),
                }
            }
        }).unwrap_or(("0".to_string(), "Type dollar amount.".to_string()));

        AmountInputContent (
            Column::new(16.0, Offset::Center, Size::Fit, Padding(16.0, 64.0, 16.0, 64.0)),
            Display::new(ctx, &num),
            SubText::new(ctx, &sub), 
            (0.0, 0.0), 0.0, 0.0 // min amount, max amount, btc, btc price
        )
    }

    pub fn validate(&mut self, ctx: &mut Context) {
        // Parse final amount as f64 for validation
        let t_formatted = self.1.2.text().spans[0].text.clone();
        let value = t_formatted.replace(",", "").parse::<f32>().unwrap_or(0.0);

        // Display subtext or error message based on parsed value
        println!("VAL: {:?}", value);
        match t_formatted.as_str() {
            "0" | "0." | "0.0" | "0.00" if !IS_MOBILE => {
                self.2.set_subtext(ctx, "Type dollar amount."); // Prompt input
                self.2.3 = true; // Disable buttons
            }
            _ if value < self.3.0 => {
                let error = format!("${:.2} minimum.", self.3.0);
                self.2.set_error(ctx, &error); // Exceeds min -> show error
                self.2.3 = true; // Disable buttons
            }
            _ if value > self.3.1 => {
                let max = self.3.1.max(0.00);
                let error = format!("${:.2} maximum.", max);
                self.2.set_error(ctx, &error); // Exceeds max -> show error
                self.2.3 = true; // Disable buttons
            }
            _ => {
                self.4 = value/self.5;
                let amount = format!("{:.8} BTC", self.4); // value divided by bitcoin price.
                self.2.set_subtext(ctx, &amount);
                self.2.3 = false; // Enable buttons
            }
        }
    }
}

impl OnEvent for AmountInputContent {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(KeyboardEvent{state: KeyboardState::Pressed, key}) = event.downcast_ref() {
            // Get font sizes from theme
            let font_size = ctx.theme.fonts.size;
            // Remove commas from input string
            let mut t = self.1.amount().text.replace(",", "");
            // Count digits (excluding dots and commas)
            let mut digit_count = t.chars().filter(|ch| ch.is_ascii_digit()).count();

            // Handle key input
            match key {
                // Handle delete or backspace when input isn't empty
                Key::Named(NamedKey::Delete | NamedKey::Backspace) if !t.is_empty() => match t.as_str() {
                    // Do nothing if the value is "$0"
                    "$0" => {},
                    // Reset to "0" if only one character remains
                    _ if t.len() == 1 => t = "0".to_string(),
                    // Remove the last character
                    _ => { t.pop(); }
                },
                // Handle regular character input
                Key::Character(c) => {
                    // Only process if digit count is under the 8-digit cap
                    if digit_count < 8 {
                        // Get the first character from input
                        let character = c.chars().next().unwrap();

                        // Only continue if character is a number
                        if character.is_ascii_digit() {
                            match t.find('.') {
                                // Allow up to 2 digits after the decimal
                                Some(i) if t[i + 1..].len() < 2 => t.push(character),
                                None => match t.as_str() {
                                    // Replace "0" with typed digit
                                    "0" => t = character.to_string(),
                                    // Append digit to current value
                                    _ => t.push(character),
                                },
                                // Do nothing if 2 decimal digits already present
                                _ => {}
                            }
                        } else if character == '.' && !t.contains('.') && digit_count + 2 <= 8 {
                            // Allow decimal if not already present and projected total digits is less than 8
                            t.push('.');
                        }
                    }
                }
                // Ignore other key types
                _ => {return true}
            }

            // Set placeholder zeroes after the decimal point
            self.1.zeros().text = match t.find('.') {
                Some(i) => match t[i + 1..].len() {
                    0 => "00",
                    1 => "0",
                    _ => "",
                },
                None => "",
            }.to_string();

            // Recalculate digit count (excluding dot/comma)
            digit_count = t.chars().filter(|ch| ch.is_ascii_digit()).count();

            // Split into dollar and cent portions
            let (dollars, cents) = match t.find('.') {
                Some(i) => (&t[..i], Some(&t[i..])),
                None => (t.as_str(), None),
            };

            // Format dollar portion with commas every 3 chars
            let formatted_dollars: String = dollars
                .chars().rev().enumerate()
                .flat_map(|(i, ch)| if i > 0 && i % 3 == 0 { vec![',', ch] } else { vec![ch] })
                .collect::<Vec<_>>().into_iter().rev().collect();

            // Combine formatted dollars and cents into final string
            let t_formatted = format!("{}{}", formatted_dollars, cents.unwrap_or(""));

            // Choose font size based on total digits (including decimal placeholders)
            let total_digits = digit_count + self.1.zeros().text.len();
            let size = if total_digits <= 5 { font_size.title } else { font_size.h1 };

            // Set final text
            self.1.amount().text = t_formatted.clone();

            // Apply font size and line height to amount and zeros and currency symbol
            self.1.amount().font_size = size;
            self.1.amount().line_height = size * 1.25;
            self.1.zeros().font_size = size;
            self.1.zeros().line_height = size * 1.25;
            self.1.currency().font_size = size;
            self.1.currency().line_height = size * 1.25;

            self.validate(ctx)
        }  
        true
    }
}

#[derive(Debug, Component)]
struct Display(Row, Text, Text, Text);
impl OnEvent for Display {}

impl Display {
    pub fn new(ctx: &mut Context, num: &str) -> Self {
        let theme = &ctx.theme;
        let font_size = theme.fonts.size.title;
        let (mc, dc) = (theme.colors.text.heading, theme.colors.text.secondary);

        Display (
            Row::center(0.0),
            Text::new(ctx, "$", TextStyle::Label(mc), font_size, Align::Left),
            Text::new(ctx, num, TextStyle::Label(mc), font_size, Align::Left),
            Text::new(ctx, "", TextStyle::Label(dc), font_size, Align::Left),
        )
    }

    pub fn value(&mut self) -> String {self.amount().text.clone()+"."+&self.zeros().text}
    pub fn amount(&mut self) -> &mut Span {&mut self.2.text().spans[0]}
    pub fn zeros(&mut self) -> &mut Span {&mut self.3.text().spans[0]}
    pub fn currency(&mut self) -> &mut Span {&mut self.1.text().spans[0]}
}
