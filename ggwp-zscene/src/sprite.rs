use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use ggez::graphics::{self, Point2, Rect};
use ggez::{Context, GameResult};

#[derive(Debug, Clone)]
struct SpriteData {
    image: graphics::Image,
    basic_scale: f32,
    param: graphics::DrawParam,
}

#[derive(Debug, Clone)]
pub struct Sprite {
    data: Rc<RefCell<SpriteData>>,
}

impl Sprite {
    pub fn from_image(image: graphics::Image, height: f32) -> Self {
        let basic_scale = height / image.height() as f32;
        println!("basic_scale: {}", basic_scale);
        let param = graphics::DrawParam {
            scale: Point2::new(basic_scale, basic_scale),
            ..Default::default()
        };
        let data = SpriteData {
            image,
            param,
            basic_scale,
        };
        let data = Rc::new(RefCell::new(data));
        Self { data }
    }

    pub fn from_path<P: AsRef<Path>>(
        context: &mut Context,
        path: P,
        height: f32,
    ) -> GameResult<Self> {
        let image = graphics::Image::new(context, path)?;
        Ok(Self::from_image(image, height))
    }

    // TODO: some method to change the image.

    pub fn set_centered(&mut self, is_centered: bool) {
        let offset = if is_centered {
            let n = 0.5;
            // Point2::new(0.5, 0.5)
            Point2::new(n, n)
        } else {
            Point2::new(0.0, 0.0)
        };
        self.set_offset(offset);
    }

    /// [0.0 .. 1.0]
    pub fn set_offset(&mut self, offset: Point2) {
        let mut data = self.data.borrow_mut();
        data.param.offset = offset;
    }

    pub fn draw(&self, context: &mut Context) -> GameResult<()> {
        let data = self.data.borrow();
        graphics::draw_ex(context, &data.image, data.param)
    }

    pub fn pos(&self) -> Point2 {
        let data = self.data.borrow();
        data.param.dest
    }

    pub fn rect(&self) -> Rect {
        let pos = self.pos();
        let data = self.data.borrow();
        let r = data.image.get_dimensions();
        // TODO: angle?
        // TODO: data.param.offset?
        // TODO: does ggez some aabb method for this?
        Rect {
            x: pos.x,
            y: pos.y,
            w: r.w * data.param.scale.x,
            h: r.h * data.param.scale.y,
        }
    }

    pub fn color_opt(&self) -> Option<graphics::Color> {
        self.data.borrow().param.color
    }

    /// NOTE: panics if the sprite has no color.
    pub fn color(&self) -> graphics::Color {
        self.color_opt().unwrap()
    }

    pub fn scale(&self) -> Point2 {
        let data = self.data.borrow();
        data.param.scale / data.basic_scale
    }

    pub fn set_pos(&mut self, pos: Point2) {
        let mut data = self.data.borrow_mut();
        // data.param.dest = pos + data.offset;
        data.param.dest = pos;
    }

    pub fn set_color(&mut self, color: graphics::Color) {
        self.data.borrow_mut().param.color = Some(color);
    }

    pub fn set_scale_h(&mut self, scale: f32) {
        // TODO: call set_scale
        let mut data = self.data.borrow_mut();
        let s = data.basic_scale * scale;
        let scale = Point2::new(s, s);
        data.param.scale = scale;
    }

    pub fn set_scale(&mut self, scale: Point2) {
        let mut data = self.data.borrow_mut();
        data.param.scale = scale * data.basic_scale;
    }

    // TODO: unittest this?
    pub fn is_same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
    }
}
