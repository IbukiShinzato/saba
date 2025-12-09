// use crate::error::Error;
// use crate::renderer::dom::node::ElementKind;
// use crate::renderer::dom::node::Node;
// use crate::renderer::dom::node::NodeKind;
// use alloc::format;
// use alloc::rc::Rc;
// use alloc::string::String;
// use alloc::string::ToString;
// use core::cell::RefCell;

// // 計算値を表す構造体
// // Task: これ以外のCSSのプロパティの実装
// #[derive(Debug, Clone, PartialEq)]
// pub struct ComputedStyle {
//     background_color: Option<Color>,
//     color: Option<Color>,
//     display: Option<DisplayType>,
//     font_size: Option<FontSize>,
//     text_decoration: Option<TextDecoretaion>,
//     height: Option<f64>,
//     width: Option<f64>,
// }

// impl ComputedStyle {
//     pub fn new() -> Self {
//         Self {
//             background_color: None,
//             color: None,
//             display: None,
//             font_size: None,
//             text_decoration: None,
//             height: None,
//             width: None,
//         }
//     }

//     pub fn set_background_color(&mut self, color: Color) {
//         self.background_color = Some(color);
//     }

//     pub fn background_color(&self) -> Color {
//         self.background_color
//             .clone()
//             .expect("failed to access CSS property: background_color")
//     }

//     pub fn set_color(&mut self, color: Color) {
//         self.color = Some(color);
//     }

//     pub fn color(&self) -> Color {
//         self.color
//             .clone()
//             .expect("failed to access CSS property: color")
//     }

//     pub fn set_display(&mut self, display: DisplayType) {
//         self.display = Some(display);
//     }

//     pub fn display(&self) -> DisplayType {
//         self.display
//             .expect("failed to access CSS property: display")
//     }

//     pub fn font_size(&self) -> FontSize {
//         self.font_size
//             .expect("failed to access CSS property: font_size")
//     }

//     pub fn text_decoration(&self) -> TextDecoration {
//         self.text_decoration
//             .expect("failed to access CSS property: text_decoration")
//     }

//     pub fn set_height(&self, height: f64) {
//         self.height = Some(height);
//     }

//     pub fn height(&self) -> f64 {
//         self.height.expect("failed to access CSS property: height")
//     }

//     pub fn set_width(&mut self, width: f64) {
//         self.width = Some(width);
//     }

//     pub fn width(&self) -> f64 {
//         self.width.expect("failed to access CSS property: width")
//     }
// }

// // CSSの色の値を表す構造体
// // 名前とカラーコードを表す値をフィールドに持つ
// #[derive(Debug, Clone, PartialEq)]
// pub struct Color {
//     name: Option<String>,
//     code: String,
// }

// // Task: 新しい色の実装
// impl Color {
//     // name -> color code
//     pub fn from_name(name: &str) -> Result<Self, Error> {
//         let code = match name {
//             "black" => "#000000".to_string(),
//             "silver" => "#c0c0c0".to_string(),
//             "gray" => "#808080".to_string(),
//             "white" => "#ffffff".to_string(),
//             "maroon" => "#800000".to_string(),
//             "red" => "#ff0000".to_string(),
//             "purple" => "#800080".to_string(),
//             "fuchsia" => "#ff00ff".to_string(),
//             "green" => "#008000".to_string(),
//             "lime" => "#00ff00".to_string(),
//             "olive" => "#808000".to_string(),
//             "yellow" => "#ffff00".to_string(),
//             "navy" => "#000080".to_string(),
//             "blue" => "#0000ff".to_string(),
//             "teal" => "#008080".to_string(),
//             "aqua" => "#00ffff".to_string(),
//             "orange" => "#ffa500".to_string(),
//             "lightgray" => "#d3d3d3".to_string(),
//             _ => {
//                 return Err(Error::UnexpectedInput(format!(
//                     "color name {:?} is not suppored yet",
//                     name
//                 )));
//             }
//         };

//         Ok(Self {
//             name: Some(name.to_string()),
//             code,
//         })
//     }

//     // color code -> name
//     pub fn from_code(code: &str) -> Result<Self, Error> {
//         //
//         if code.chars().nth(0) != Some('#') || code.len() != 7 {
//             return Err(Error::UnexpectedInput(format!(
//                 "invalid color code {}",
//                 code
//             )));
//         }

//         let name = match code.to_lowercase().as_str() {
//             "#000000" => "black",
//             "#c0c0c0" => "silver",
//             "#808080" => "gray",
//             "#ffffff" => "white",
//             "#800000" => "maroon",
//             "#ff0000" => "red",
//             "#800080" => "purple",
//             "#ff00ff" => "fuchsia",
//             "#008000" => "green",
//             "#00ff00" => "lime",
//             "#808000" => "olive",
//             "#ffff00" => "yellow",
//             "#000080" => "navy",
//             "#0000ff" => "blue",
//             "#008080" => "teal",
//             "#00ffff" => "aqua",
//             "#ffa500" => "orange",
//             "#d3d3d3" => "lightgray",
//             _ => {
//                 return Err(Error::UnexpectedInput(format!(
//                     "color code {} is not supported yet",
//                     code
//                 )));
//             }
//         };

//         Ok(Self {
//             name: Some(name.to_string()),
//             code: code.to_string(),
//         })
//     }

//     // whiteを表すColorオブジェクトを生成
//     pub fn white() -> Self {
//         Self {
//             name: Some("white".to_string()),
//             code: "#ffffff".to_string(),
//         }
//     }

//     // blackを表すColorオブジェクトを生成
//     pub fn black() -> Self {
//         Self {
//             name: Some("black".to_string()),
//             code: "#000000".to_string(),
//         }
//     }

//     // color code を u32型で返す
//     pub fn code_u32(&self) -> u32 {
//         // '#'は取り除いてu32に変換
//         u32::from_str_radix(self.code.trim_matches('#'), 16).unwrap()
//     }
// }

// // 文字の大きさを表す列挙型
// // h1がXXLarge, h2がXLarge, 通常の文字がMedium
// #[derive(Debug, Copy, Clone, PartialEq)]
// pub enum FontSize {
//     Medium,
//     XLarge,
//     XXLarge,
// }

// impl FontSize {
//     fn default(node: &Rc<RefCell<Node>>) -> Self {
//         match &node.borrow().kind() {
//             NodeKind::Element(element) => match element.kind() {
//                 ElementKind::H1 => FontSize::XXLarge,
//                 ElementKind::H2 => FontSize::XLarge,
//                 _ => FontSize::Medium,
//             },
//             _ => FontSize::Medium,
//         }
//     }
// }

// // CSSの displayプロパティに対応する値を表す
// #[derive(Debug, Copy, Clone, PartialEq)]
// pub enum DisplayType {
//     Block,
//     Inline,
//     DisplayNone,
// }

// impl DisplayType {
//     fn default(node: &Rc<RefCell<Node>>) -> Self {
//         match &node.borrow().kind() {
//             NodeKind::Document => DisplayType::Block,
//             NodeKind::Element(e) => {
//                 if e.is_block_element() {
//                     DisplayType::Block
//                 } else {
//                     DisplayType::Inline
//                 }
//             }
//             NodeKind::Text(_) => DisplayType::Inline,
//         }
//     }
// }
